//! Checks on the published HTTP contract. They need no database: they read the
//! same route registry the server mounts, so they fail the moment a route is
//! added or changed in a way that breaks one of these rules.

use forklaunch_core::contract::{AccessLevel, RouteContract};

fn routes() -> Vec<RouteContract> {
    api::sdk::registry().routes().to_vec()
}

fn route(operation: &str) -> RouteContract {
    routes()
        .into_iter()
        .find(|route| route.name == operation)
        .unwrap_or_else(|| panic!("no route named {operation}"))
}

fn allowed_roles(route: &RouteContract) -> Vec<String> {
    route.rbac.as_ref().map(|rbac| rbac.allowed_roles.clone()).unwrap_or_default()
}

fn success_schemas(route: &RouteContract) -> String {
    route
        .responses
        .iter()
        .filter(|response| (200..300).contains(&response.status_code))
        .filter_map(|response| response.body.as_ref()?.schema.json_schema.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn only_sign_in_and_health_probes_are_public() {
    let mut public: Vec<String> = routes()
        .into_iter()
        .filter(|route| route.access == AccessLevel::Public)
        .map(|route| route.path)
        .collect();
    public.sort();
    assert_eq!(public, ["/api/auth/login", "/api/healthz", "/api/readyz", "/health"]);
}

#[test]
fn every_business_route_declares_the_roles_that_may_call_it() {
    for route in routes() {
        let business = ["/api/applications", "/api/documents", "/api/users"]
            .iter()
            .any(|prefix| route.path.starts_with(prefix));
        if business {
            assert_eq!(
                route.access,
                AccessLevel::Protected,
                "{} must be role-protected",
                route.path
            );
            assert!(!allowed_roles(&route).is_empty(), "{} declares no roles", route.path);
        }
    }
}

#[test]
fn role_rules_match_the_product() {
    let lending = ["relationship_manager", "credit_officer"];
    // Admins manage users and never see borrower data.
    for route in routes().iter().filter(|route| {
        route.path.starts_with("/api/applications") || route.path.starts_with("/api/documents")
    }) {
        assert!(
            !allowed_roles(route).contains(&"admin".to_owned()),
            "{} admits admins",
            route.path
        );
    }
    for route in routes().iter().filter(|route| route.path.starts_with("/api/users")) {
        assert_eq!(allowed_roles(route), ["admin"], "{}", route.path);
    }
    assert_eq!(allowed_roles(&route("Create Application")), ["relationship_manager"]);
    assert_eq!(allowed_roles(&route("Request Document Upload")), ["relationship_manager"]);
    // Only credit officers decide; relationship managers cannot approve anything,
    // their own applications included. (The creator rule is enforced per record.)
    assert_eq!(allowed_roles(&route("Decide Application")), ["credit_officer"]);
    assert_eq!(allowed_roles(&route("Reveal Owner SSN")), lending);
}

#[test]
fn a_full_ssn_leaves_the_server_only_through_the_audited_reveal_endpoint() {
    for route in routes() {
        let schemas = success_schemas(&route);
        let has_ssn = schemas.contains("\"ssn\"");
        assert_eq!(
            has_ssn,
            route.name == "Reveal Owner SSN",
            "{} {} full-SSN field",
            route.path,
            if has_ssn { "exposes a" } else { "is missing its" }
        );
        for forbidden in
            ["ssn_ciphertext", "ssn_encrypted", "password_hash", "password\"", "secret_ciphertext"]
        {
            assert!(!schemas.contains(forbidden), "{} returns {forbidden}", route.path);
        }
    }
    // Screens get the last four digits only.
    assert!(success_schemas(&route("Get Application")).contains("ssn_last4"));
    assert!(success_schemas(&route("Export Application")).contains("ssn_last4"));
}

#[test]
fn nothing_sensitive_is_carried_in_a_url() {
    // Paths carry opaque ids only; query strings carry paging only. Search
    // text (EINs, names) and every other value travel in the request body.
    for route in routes() {
        let mut query: Vec<&str> =
            route.request.query.iter().map(|field| field.name.as_str()).collect();
        query.sort_unstable();
        assert!(
            query.iter().all(|name| ["limit", "cursor"].contains(name)),
            "{} takes {query:?} in its query string",
            route.path
        );
        for parameter in &route.request.params {
            assert!(
                ["id", "owner_id"].contains(&parameter.name.as_str()),
                "{} has path parameter {}",
                route.path,
                parameter.name
            );
        }
    }
    assert_eq!(route("Search Applications").method.as_str(), "POST");
}

#[test]
fn list_endpoints_are_paginated() {
    for operation in ["List Applications", "Search Applications", "List Users"] {
        let route = route(operation);
        let headers: Vec<&str> = route
            .responses
            .iter()
            .filter(|response| response.status_code == 200)
            .flat_map(|response| response.headers.iter().map(|header| header.name.as_str()))
            .collect();
        assert!(headers.contains(&"x-next-cursor"), "{operation} does not return a page cursor");
    }
    let query: Vec<String> =
        route("List Applications").request.query.iter().map(|field| field.name.clone()).collect();
    assert!(query.contains(&"limit".to_owned()) && query.contains(&"cursor".to_owned()));
}

#[test]
fn creating_an_application_accepts_an_idempotency_key() {
    let headers: Vec<String> = route("Create Application")
        .request
        .headers
        .iter()
        .map(|field| field.name.clone())
        .collect();
    assert_eq!(headers, ["idempotency-key"]);
}
