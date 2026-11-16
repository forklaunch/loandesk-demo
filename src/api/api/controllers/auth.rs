//! Sign-in, sign-out, the current user, and two-factor enrolment.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::auth::{
        ChangePassword, LoginRequest, MfaCode, MfaEnrolment, Ok as OkBody, SessionUser,
    },
    error::AppError,
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};

pub async fn login(
    State(state): State<AppState>,
    input: LoginRequest,
) -> Result<SessionUser, EndpointError> {
    state.auth.login(input).await.map_err(AppError::into_endpoint)
}
pub async fn logout(
    State(state): State<AppState>,
    user: CurrentUser,
    (): (),
) -> Result<OkBody, EndpointError> {
    state.auth.logout(&user).await.map_err(AppError::into_endpoint)
}
pub async fn me(
    State(_): State<AppState>,
    user: CurrentUser,
    (): (),
) -> Result<SessionUser, EndpointError> {
    Ok(user.to_session_user())
}
pub async fn mfa_enrol(
    State(state): State<AppState>,
    user: CurrentUser,
    (): (),
) -> Result<MfaEnrolment, EndpointError> {
    state.auth.mfa_enrol(&user).await.map_err(AppError::into_endpoint)
}
pub async fn mfa_confirm(
    State(state): State<AppState>,
    user: CurrentUser,
    input: MfaCode,
) -> Result<OkBody, EndpointError> {
    state.auth.mfa_confirm(&user, &input.code).await.map_err(AppError::into_endpoint)
}
pub async fn change_password(
    State(state): State<AppState>,
    user: CurrentUser,
    input: ChangePassword,
) -> Result<OkBody, EndpointError> {
    state.auth.change_password(&user, input).await.map_err(AppError::into_endpoint)
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::auth::{
        ChangePassword as ChangePasswordBody, LoginRequest, MfaCode, MfaEnrolment, Ok, SessionUser,
    };
    use forklaunch_core::contract::{
        HttpMethod,
        endpoint::{Endpoint, JsonBody, NoBody},
    };

    #[access(Public)]
    pub fn login() -> Endpoint<JsonBody<LoginRequest>, JsonBody<SessionUser>> {
        Endpoint::new(
            "Sign In",
            "Verify email, password and (when enabled) a two-factor code; starts a session",
            HttpMethod::Post,
            "/api/auth/login",
            200,
        )
        .expect("valid login endpoint")
    }
    #[access(Authenticated)]
    pub fn logout() -> Endpoint<NoBody, JsonBody<Ok>> {
        Endpoint::new(
            "Sign Out",
            "End the caller's session",
            HttpMethod::Post,
            "/api/auth/logout",
            200,
        )
        .expect("valid logout endpoint")
    }
    #[access(Authenticated)]
    pub fn me() -> Endpoint<NoBody, JsonBody<SessionUser>> {
        Endpoint::new("Current User", "The signed-in user", HttpMethod::Get, "/api/auth/me", 200)
            .expect("valid me endpoint")
    }
    #[access(Authenticated)]
    pub fn mfa_enrol() -> Endpoint<NoBody, JsonBody<MfaEnrolment>> {
        Endpoint::new(
            "Start Two-Factor Enrolment",
            "Issue a new authenticator secret for the caller",
            HttpMethod::Post,
            "/api/auth/mfa/enrol",
            200,
        )
        .expect("valid mfa enrol endpoint")
    }
    #[access(Authenticated)]
    pub fn mfa_confirm() -> Endpoint<JsonBody<MfaCode>, JsonBody<Ok>> {
        Endpoint::new(
            "Confirm Two-Factor Enrolment",
            "Turn two-factor sign-in on once a code from the authenticator verifies",
            HttpMethod::Post,
            "/api/auth/mfa/confirm",
            200,
        )
        .expect("valid mfa confirm endpoint")
    }
    #[access(Authenticated)]
    pub fn change_password() -> Endpoint<JsonBody<ChangePasswordBody>, JsonBody<Ok>> {
        Endpoint::new(
            "Change Password",
            "Change the caller's own password; ends every session of the user and starts a new one",
            HttpMethod::Post,
            "/api/auth/password",
            200,
        )
        .expect("valid change password endpoint")
    }
}
