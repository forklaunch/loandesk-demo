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
fn every_endpoint_requires_sign_in_unless_it_is_explicitly_public() {
    let mut public: Vec<String> = routes()
        .into_iter()
        .filter(|route| route.access == AccessLevel::Public)
        .map(|route| route.path)
        .collect();
    public.sort();
    assert_eq!(public, ["/api/auth/login", "/api/healthz", "/api/readyz", "/health"]);
    // Everything else is bound to the session policy: `Authenticated` (any
    // signed-in user) or `Protected` (signed in, with one of the listed roles).
    for route in routes().iter().filter(|route| route.access != AccessLevel::Public) {
        assert!(
            matches!(route.access, AccessLevel::Authenticated | AccessLevel::Protected),
            "{} has an unexpected access level",
            route.path
        );
    }
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
    for operation in ["List Applications", "Search Applications", "List Users", "List Audit Log"] {
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

#[test]
fn browser_error_reports_need_a_session_and_have_no_field_for_the_message() {
    let route = route("Report Browser Error");
    assert_eq!(route.path, "/api/client-errors");
    assert_ne!(route.access, AccessLevel::Public);
    let body = route
        .request
        .body
        .as_ref()
        .and_then(|body| body.schema.json_schema.clone())
        .expect("the report has a declared body");
    for field in ["kind", "class", "screen", "stack", "release"] {
        assert!(body.contains(&format!("\"{field}\"")), "missing {field}");
    }
    // An error message can quote what a user typed, so it cannot be sent at all.
    assert!(!body.contains("\"message\""), "{body}");
    assert!(body.contains("\"additionalProperties\":false"), "{body}");
}

/// The JSON schema of a route's request body, if it has one.
fn request_schema(route: &RouteContract) -> Option<serde_json::Value> {
    let schema = route.request.body.as_ref()?.schema.json_schema.clone()?;
    Some(serde_json::from_str(&schema).expect("request schema is JSON"))
}

/// Every object schema nested in `schema`, with the path that leads to it.
fn objects<'a>(
    schema: &'a serde_json::Value,
    path: String,
    out: &mut Vec<(String, &'a serde_json::Value)>,
) {
    match schema {
        serde_json::Value::Object(map) => {
            if map.contains_key("properties") {
                out.push((path.clone(), schema));
            }
            for (key, value) in map {
                objects(value, format!("{path}/{key}"), out);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                objects(value, format!("{path}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// (route name, property name, property schema) of every request body field.
fn request_fields() -> Vec<(String, String, serde_json::Value)> {
    let mut fields = Vec::new();
    for route in routes() {
        let Some(schema) = request_schema(&route) else { continue };
        let mut found = Vec::new();
        objects(&schema, String::new(), &mut found);
        for (_, object) in found {
            for (name, property) in object["properties"].as_object().expect("properties") {
                fields.push((route.name.clone(), name.clone(), property.clone()));
            }
        }
    }
    assert!(
        fields.len() >= 25,
        "found only {} request fields; is the scan still looking?",
        fields.len()
    );
    fields
}

#[test]
fn unknown_fields_in_requests_are_rejected() {
    // Every request body, and every object nested in one, is a closed schema:
    // a field the contract does not declare is refused with 400 before the
    // handler runs (scripts/e2e.py sends one to check).
    let mut bodies = 0;
    for route in routes() {
        let Some(schema) = request_schema(&route) else { continue };
        let mut found = Vec::new();
        objects(&schema, String::new(), &mut found);
        assert!(!found.is_empty(), "{} has a body without declared fields", route.name);
        for (path, object) in found {
            bodies += 1;
            assert_eq!(
                object.get("additionalProperties"),
                Some(&serde_json::Value::Bool(false)),
                "{} accepts undeclared fields at {path}",
                route.name
            );
        }
    }
    assert!(bodies >= 11, "checked only {bodies} request objects");
}

#[test]
fn strings_in_input_have_length_limits() {
    for (route, name, property) in request_fields() {
        if property["type"] == "string" {
            assert!(
                property["maxLength"].as_u64().is_some_and(|max| max <= 4000),
                "{route}: `{name}` has no maximum length"
            );
        }
        if property["type"] == "integer" {
            assert!(
                property.get("maximum").is_some() && property.get("minimum").is_some(),
                "{route}: `{name}` has no numeric bounds"
            );
        }
    }
    // Query strings and headers too.
    for route in routes() {
        for field in route.request.query.iter().chain(route.request.headers.iter()) {
            let schema = field.schema.json_schema.clone().unwrap_or_default();
            assert!(
                schema.contains("maxLength") || schema.contains("maximum"),
                "{}: `{}` is unbounded ({schema})",
                route.path,
                field.name
            );
        }
    }
    // The one array a request can carry (an application's owners) has no
    // schema-level item limit: the framework's schema macro has no option for
    // it. `ApplicationsService::create` refuses fewer than 1 or more than 20
    // (scripts/e2e.py sends 21), and the body as a whole is capped at 2 MiB.
    let arrays: Vec<(String, String)> = request_fields()
        .into_iter()
        .filter(|(_, _, property)| property["type"] == "array")
        .map(|(route, name, _)| (route, name))
        .collect();
    assert_eq!(arrays, [("Create Application".to_owned(), "owners".to_owned())]);
}

#[test]
fn clients_cannot_set_privileged_fields_through_mass_assignment() {
    // Fields the server decides. None of them is accepted in any request body
    // (and unknown fields are refused, see above), so a client cannot name
    // its own role, tenant, owner, decision or status.
    let server_side = [
        "id",
        "tenant_id",
        "tenant",
        "created_by",
        "created_at",
        "decided_by",
        "decided_at",
        "status",
        "reference",
        "password_hash",
        "failed_logins",
        "locked_until",
        "disabled_at",
        "disabled_reason",
        "last_login_at",
        "uploaded_by",
        "storage_key",
        "classification",
        "search_tokens",
        "mfa_enabled",
        "user_id",
    ];
    for (route, name, _) in request_fields() {
        assert!(!server_side.contains(&name.as_str()), "{route} accepts `{name}` from the client");
    }
    // `role` is accepted in exactly two places, both admin-only.
    let mut role_routes: Vec<String> = request_fields()
        .into_iter()
        .filter(|(_, name, _)| name == "role")
        .map(|(route, _, _)| route)
        .collect();
    role_routes.sort();
    assert_eq!(role_routes, ["Create User", "Set User Role"]);
    for name in role_routes {
        assert_eq!(allowed_roles(&route(&name)), ["admin"]);
    }
}

#[test]
fn risk_figures_come_from_authoritative_application_data() {
    // Revenue, net income, total debt and the risk summary are written by the
    // worker from the uploaded documents and computed by `shared::risk` on
    // every read. No request can supply or replace them.
    let derived = [
        "revenue",
        "net_income",
        "total_debt",
        "risk",
        "dscr",
        "leverage",
        "annual_debt_service",
        "flag",
    ];
    for (route, name, _) in request_fields() {
        assert!(!derived.contains(&name.as_str()), "{route} accepts `{name}` from the client");
    }
    // They are returned, read-only, with the application.
    let view = success_schemas(&route("Get Application"));
    for field in ["revenue", "net_income", "total_debt", "dscr", "leverage"] {
        assert!(view.contains(&format!("\"{field}\"")), "the application view lost `{field}`");
    }
}

#[test]
fn the_tenant_id_comes_from_the_deployment_not_from_the_request() {
    // One trusted tenant per deployment (TENANT_ID), bound to every database
    // transaction (`Db::tx`) and checked by forced row-level security. No
    // route takes a tenant in its path, query, headers or body.
    for route in routes() {
        let mut names: Vec<String> = route
            .request
            .params
            .iter()
            .chain(route.request.query.iter())
            .chain(route.request.headers.iter())
            .map(|field| field.name.to_lowercase())
            .collect();
        if let Some(schema) = request_schema(&route) {
            let mut found = Vec::new();
            objects(&schema, String::new(), &mut found);
            for (_, object) in found {
                names.extend(object["properties"].as_object().expect("properties").keys().cloned());
            }
        }
        assert!(!names.iter().any(|name| name.contains("tenant")), "{} takes a tenant", route.path);
    }
}

#[test]
fn public_and_expensive_endpoints_are_rate_limited() {
    use api::api::{rate_limit::EXPENSIVE_OPERATIONS, transport::LOGIN_PATH};
    // The one public route that does work is limited per client address.
    let public: Vec<String> = routes()
        .into_iter()
        .filter(|route| route.access == AccessLevel::Public && !route.path.contains("health"))
        .filter(|route| route.path != "/api/readyz")
        .map(|route| route.path)
        .collect();
    assert_eq!(public, [LOGIN_PATH]);
    // Every other route goes through the session policy, which applies the
    // per-user limit; the expensive ones have a second, lower limit. Each
    // name in that list must be a real route, or the limit would never apply.
    for operation in EXPENSIVE_OPERATIONS {
        assert_ne!(route(operation).access, AccessLevel::Public, "{operation}");
    }
    // Anything that searches, exports, reveals, signs a grant or hashes a
    // password is on the list.
    for route in routes() {
        let costly = ["search", "export", "reveal", "upload", "download", "password"]
            .iter()
            .any(|word| route.path.contains(word));
        if costly {
            assert!(
                EXPENSIVE_OPERATIONS.contains(&route.name.as_str()),
                "{} ({}) is not rate limited as an expensive operation",
                route.name,
                route.path
            );
        }
    }
}

#[test]
fn a_person_s_data_can_be_exported_in_a_machine_readable_format() {
    // One application with its owners and documents, as JSON, for the people
    // who may see it; audited (`application_exported`).
    let export = route("Export Application");
    assert_eq!(export.method.as_str(), "GET");
    assert_eq!(allowed_roles(&export), ["relationship_manager", "credit_officer"]);
    let schema = success_schemas(&export);
    for field in ["exported_at", "application", "owners", "documents", "ssn_last4", "date_of_birth"]
    {
        assert!(schema.contains(&format!("\"{field}\"")), "the export lost `{field}`");
    }
}

#[test]
fn access_review_and_account_lifecycle_routes_are_admin_only() {
    for operation in
        ["Access Review", "Record Access Review", "Disable User", "Enable User", "List Audit Log"]
    {
        assert_eq!(allowed_roles(&route(operation)), ["admin"], "{operation}");
    }
    // Changing a password needs a session, and only ever changes the caller's own.
    let password = route("Change Password");
    assert_eq!(password.access, AccessLevel::Authenticated);
    assert!(password.request.params.is_empty(), "no user id in the path: it is the caller's");
}
