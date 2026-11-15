//! User administration. Every route is restricted to the `admin` role in its contract.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        users::{NewUser, SetRole, User},
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

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{
        IdPath, PageHeaders, PageQuery,
        users::{NewUser, SetRole as SetRoleBody, User},
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
}
