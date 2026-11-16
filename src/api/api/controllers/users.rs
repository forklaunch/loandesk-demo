//! User administration. Every route is restricted to the `admin` role in its contract.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        users::{AccessReview, AccessReviewRecorded, AuditEntry, NewUser, SetRole, User},
    },
    error::AppError,
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};
use forklaunch_core::contract::endpoint::{NoFields, RequestFields, ResponseWithHeaders};

pub async fn list_users(
    State(state): State<AppState>,
    _admin: CurrentUser,
    request: RequestFields<(), PageQuery, NoFields, NoFields>,
) -> Result<ResponseWithHeaders<Vec<User>, PageHeaders>, EndpointError> {
    let (body, next_cursor) =
        state.users.list(request.query).await.map_err(AppError::into_endpoint)?;
    Ok(ResponseWithHeaders { body, headers: PageHeaders { next_cursor } })
}
pub async fn create_user(
    State(state): State<AppState>,
    admin: CurrentUser,
    input: NewUser,
) -> Result<User, EndpointError> {
    state.users.create(&admin, input).await.map_err(AppError::into_endpoint)
}
pub async fn set_role(
    State(state): State<AppState>,
    admin: CurrentUser,
    request: RequestFields<SetRole, NoFields, NoFields, IdPath>,
) -> Result<User, EndpointError> {
    state
        .users
        .set_role(&admin, &request.path.id, &request.body.role)
        .await
        .map_err(AppError::into_endpoint)
}
pub async fn list_audit_log(
    State(state): State<AppState>,
    _admin: CurrentUser,
    request: RequestFields<(), PageQuery, NoFields, NoFields>,
) -> Result<ResponseWithHeaders<Vec<AuditEntry>, PageHeaders>, EndpointError> {
    let (body, next_cursor) =
        state.users.audit_log(request.query).await.map_err(AppError::into_endpoint)?;
    Ok(ResponseWithHeaders { body, headers: PageHeaders { next_cursor } })
}
pub async fn disable_user(
    State(state): State<AppState>,
    admin: CurrentUser,
    request: RequestFields<(), NoFields, NoFields, IdPath>,
) -> Result<User, EndpointError> {
    state.users.set_disabled(&admin, &request.path.id, true).await.map_err(AppError::into_endpoint)
}
pub async fn enable_user(
    State(state): State<AppState>,
    admin: CurrentUser,
    request: RequestFields<(), NoFields, NoFields, IdPath>,
) -> Result<User, EndpointError> {
    state.users.set_disabled(&admin, &request.path.id, false).await.map_err(AppError::into_endpoint)
}
pub async fn access_review(
    State(state): State<AppState>,
    _admin: CurrentUser,
    (): (),
) -> Result<AccessReview, EndpointError> {
    state.users.access_review().await.map_err(AppError::into_endpoint)
}
pub async fn record_access_review(
    State(state): State<AppState>,
    admin: CurrentUser,
    (): (),
) -> Result<AccessReviewRecorded, EndpointError> {
    state.users.record_access_review(&admin).await.map_err(AppError::into_endpoint)
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        users::{
            AccessReview as AccessReviewBody, AccessReviewRecorded, AuditEntry, NewUser,
            SetRole as SetRoleBody, User,
        },
    };
    use forklaunch_core::contract::{
        HttpMethod, RbacConfig,
        endpoint::{
            Endpoint, EndpointBody, Fields, JsonBody, NoBody, NoFields, WithResponseHeaders,
        },
    };

    fn admins_only<I: EndpointBody, O: EndpointBody>(endpoint: Endpoint<I, O>) -> Endpoint<I, O> {
        Endpoint::from_contract(endpoint.into_contract().with_rbac(RbacConfig {
            allowed_roles: vec!["admin".to_owned()],
            ..RbacConfig::default()
        }))
        .expect("valid role restriction")
    }

    #[access(Protected)]
    pub fn list_users() -> Endpoint<
        Fields<NoBody, PageQuery, NoFields, NoFields>,
        WithResponseHeaders<JsonBody<Vec<User>>, PageHeaders>,
    > {
        admins_only(
            Endpoint::new(
                "List Users",
                "One page of users ordered by name; x-next-cursor continues",
                HttpMethod::Get,
                "/api/users",
                200,
            )
            .expect("valid list users endpoint"),
        )
        .with_typed_response_headers::<PageHeaders>()
        .expect("valid page headers")
    }
    #[access(Protected)]
    pub fn create_user() -> Endpoint<JsonBody<NewUser>, JsonBody<User>> {
        admins_only(
            Endpoint::new("Create User", "Add a user", HttpMethod::Post, "/api/users", 201)
                .expect("valid create user endpoint"),
        )
    }
    #[access(Protected)]
    pub fn set_role()
    -> Endpoint<Fields<JsonBody<SetRoleBody>, NoFields, NoFields, IdPath>, JsonBody<User>> {
        admins_only(
            Endpoint::new(
                "Set User Role",
                "Change another user's role; ends that user's sessions",
                HttpMethod::Patch,
                "/api/users/:id",
                200,
            )
            .expect("valid set role endpoint"),
        )
    }
    #[access(Protected)]
    pub fn disable_user() -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<User>> {
        admins_only(
            Endpoint::new(
                "Disable User",
                "Disable another user's account and end its sessions",
                HttpMethod::Post,
                "/api/users/:id/disable",
                200,
            )
            .expect("valid disable user endpoint"),
        )
    }
    #[access(Protected)]
    pub fn enable_user() -> Endpoint<Fields<NoBody, NoFields, NoFields, IdPath>, JsonBody<User>> {
        admins_only(
            Endpoint::new(
                "Enable User",
                "Re-enable a disabled account",
                HttpMethod::Post,
                "/api/users/:id/enable",
                200,
            )
            .expect("valid enable user endpoint"),
        )
    }
    #[access(Protected)]
    pub fn access_review() -> Endpoint<NoBody, JsonBody<AccessReviewBody>> {
        admins_only(
            Endpoint::new(
                "Access Review",
                "Every account with role, second factor, last sign-in and staleness, for the periodic access review",
                HttpMethod::Get,
                "/api/access-review",
                200,
            )
            .expect("valid access review endpoint"),
        )
    }
    #[access(Protected)]
    pub fn record_access_review() -> Endpoint<NoBody, JsonBody<AccessReviewRecorded>> {
        admins_only(
            Endpoint::new(
                "Record Access Review",
                "Record in the audit trail that the caller reviewed the accounts as they are now",
                HttpMethod::Post,
                "/api/access-review",
                201,
            )
            .expect("valid record access review endpoint"),
        )
    }
    #[access(Protected)]
    pub fn list_audit_log() -> Endpoint<
        Fields<NoBody, PageQuery, NoFields, NoFields>,
        WithResponseHeaders<JsonBody<Vec<AuditEntry>>, PageHeaders>,
    > {
        admins_only(
            Endpoint::new(
                "List Audit Log",
                "One page of the append-only audit trail, newest first; x-next-cursor continues",
                HttpMethod::Get,
                "/api/audit-log",
                200,
            )
            .expect("valid audit log endpoint"),
        )
        .with_typed_response_headers::<PageHeaders>()
        .expect("valid page headers")
    }
}
