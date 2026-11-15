//! Loan applications. Relationship managers see and create their own; credit
//! officers see all and decide; admins have no access (declared per route).
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        applications::{
            Application, ApplicationDetail, ApplicationExport, Created, Decided, Decision,
            IdempotencyHeaders, NewApplication, OwnerPath, RevealedSsn, SearchRequest,
        },
    },
    error::AppError,
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};
use forklaunch_core::contract::endpoint::{NoFields, RequestFields, ResponseWithHeaders};

type Page = ResponseWithHeaders<Vec<Application>, PageHeaders>;
type ById<B> = RequestFields<B, NoFields, NoFields, IdPath>;

pub async fn list_applications(
    State(state): State<AppState>,
    user: CurrentUser,
    request: RequestFields<(), PageQuery, NoFields, NoFields>,
) -> Result<Page, EndpointError> {
    let query = request.query;
    let (body, next_cursor) = state
        .applications
        .list(&user, None, query.limit.into_option(), query.cursor.into_option())
        .await
        .map_err(AppError::into_endpoint)?;
    Ok(ResponseWithHeaders { body, headers: PageHeaders { next_cursor } })
}
pub async fn search_applications(
    State(state): State<AppState>,
    user: CurrentUser,
    input: SearchRequest,
) -> Result<Page, EndpointError> {
    let (body, next_cursor) = state
        .applications
        .list(&user, Some(input.q), input.limit.into_option(), input.cursor.into_option())
        .await
        .map_err(AppError::into_endpoint)?;
    Ok(ResponseWithHeaders { body, headers: PageHeaders { next_cursor } })
}
pub async fn create_application(
    State(state): State<AppState>,
    user: CurrentUser,
    request: RequestFields<NewApplication, NoFields, IdempotencyHeaders, NoFields>,
) -> Result<Created, EndpointError> {
    state
        .applications
        .create(&user, request.body, request.headers.idempotency_key.into_option())
        .await
        .map_err(AppError::into_endpoint)
}
pub async fn get_application(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<()>,
) -> Result<ApplicationDetail, EndpointError> {
    state.applications.detail(&user, &request.path.id).await.map_err(AppError::into_endpoint)
}
pub async fn decide_application(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<Decision>,
) -> Result<Decided, EndpointError> {
    state
        .applications
        .decide(&user, &request.path.id, request.body)
        .await
        .map_err(AppError::into_endpoint)
}
pub async fn reveal_ssn(
    State(state): State<AppState>,
    user: CurrentUser,
    request: RequestFields<(), NoFields, NoFields, OwnerPath>,
) -> Result<RevealedSsn, EndpointError> {
    state
        .applications
        .reveal_ssn(&user, &request.path.id, &request.path.owner_id)
        .await
        .map_err(AppError::into_endpoint)
}
pub async fn export_application(
    State(state): State<AppState>,
    user: CurrentUser,
    request: ById<()>,
) -> Result<ApplicationExport, EndpointError> {
    state.applications.export(&user, &request.path.id).await.map_err(AppError::into_endpoint)
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        applications::{
            Application, ApplicationDetail, ApplicationExport, Created, Decided, Decision,
            IdempotencyHeaders, NewApplication, OwnerPath, RevealedSsn, SearchRequest,
        },
    };
    use forklaunch_core::contract::{
        HttpMethod, RbacConfig,
        endpoint::{
            Endpoint, EndpointBody, Fields, JsonBody, NoBody, NoFields, WithResponseHeaders,
        },
    };

    const LENDING: [&str; 2] = ["relationship_manager", "credit_officer"];

    fn roles<I: EndpointBody, O: EndpointBody>(
        endpoint: Endpoint<I, O>,
        allowed: &[&str],
    ) -> Endpoint<I, O> {
        Endpoint::from_contract(endpoint.into_contract().with_rbac(RbacConfig {
            allowed_roles: allowed.iter().map(|role| (*role).to_owned()).collect(),
            ..RbacConfig::default()
        }))
        .expect("valid role restriction")
    }

    #[access(Protected)]
    pub fn list_applications() -> Endpoint<
        Fields<NoBody, PageQuery, NoFields, NoFields>,
        WithResponseHeaders<JsonBody<Vec<Application>>, PageHeaders>,
    > {
        roles(
            Endpoint::new(
                "List Applications",
                "One page of the applications the caller may see, newest first; x-next-cursor continues",
                HttpMethod::Get,
                "/api/applications",
                200,
            )
            .expect("valid list applications endpoint"),
            &LENDING,
        )
        .with_typed_response_headers::<PageHeaders>()
        .expect("valid page headers")
    }
    #[access(Protected)]
    pub fn search_applications() -> Endpoint<
        JsonBody<SearchRequest>,
        WithResponseHeaders<JsonBody<Vec<Application>>, PageHeaders>,
    > {
        roles(
            Endpoint::new(
                "Search Applications",
                "Search the applications the caller may see; the search text is sent in the body, not the URL",
                HttpMethod::Post,
                "/api/applications/search",
                200,
            )
            .expect("valid search applications endpoint"),
            &LENDING,
        )
        .with_typed_response_headers::<PageHeaders>()
        .expect("valid page headers")
    }
    #[access(Protected)]
    pub fn create_application() -> Endpoint<
        Fields<JsonBody<NewApplication>, NoFields, IdempotencyHeaders, NoFields>,
        JsonBody<Created>,
    > {
        roles(
            Endpoint::new(
                "Create Application",
                "Create a loan application; an Idempotency-Key header makes retries safe",
                HttpMethod::Post,
                "/api/applications",
                201,
            )
            .expect("valid create application endpoint"),
            &["relationship_manager"],
        )
    }
    #[access(Protected)]
    pub fn get_application()
    -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<ApplicationDetail>> {
        roles(
            Endpoint::new(
                "Get Application",
                "One application with its owners (SSN last four only) and documents",
                HttpMethod::Get,
                "/api/applications/:id",
                200,
            )
            .expect("valid get application endpoint"),
            &LENDING,
        )
    }
    #[access(Protected)]
    pub fn decide_application()
    -> Endpoint<Fields<JsonBody<Decision>, NoFields, NoFields, IdPath>, JsonBody<Decided>> {
        roles(
            Endpoint::new(
                "Decide Application",
                "Approve or decline; never allowed for the application's creator",
                HttpMethod::Post,
                "/api/applications/:id/decision",
                200,
            )
            .expect("valid decide application endpoint"),
            &["credit_officer"],
        )
    }
    #[access(Protected)]
    pub fn reveal_ssn()
    -> Endpoint<Fields<NoBody, NoFields, NoFields, OwnerPath>, JsonBody<RevealedSsn>> {
        roles(
            Endpoint::new(
                "Reveal Owner SSN",
                "Return one owner's full SSN; every call is written to the audit trail first",
                HttpMethod::Post,
                "/api/applications/:id/owners/:owner_id/reveal-ssn",
                200,
            )
            .expect("valid reveal ssn endpoint"),
            &LENDING,
        )
    }
    #[access(Protected)]
    pub fn export_application()
    -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<ApplicationExport>> {
        roles(
            Endpoint::new(
                "Export Application",
                "Machine-readable export of one application; audited, full SSNs excluded",
                HttpMethod::Get,
                "/api/applications/:id/export",
                200,
            )
            .expect("valid export application endpoint"),
            &LENDING,
        )
    }
}
