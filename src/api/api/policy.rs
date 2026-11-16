//! Endpoint admission for every signed-in route.
//!
//! The session cookie is resolved against the database on each request, so a
//! deleted session, a changed role or a removed user takes effect on the very
//! next call. Role restrictions are declared on the route contract
//! (`allowed_roles`) and enforced here, in one place, before the body is parsed.
//! Rejected sessions, denied permissions and rate-limit refusals are logged as
//! named events (`invalid_session`, `permission_denied`, `rate_limited`).
use crate::{
    api::{rate_limit::EXPENSIVE_OPERATIONS, transport},
    domain::schemas::auth::SessionUser,
    error::AppError,
    registrations::AppState,
};
use axum::http::{HeaderMap, header, request::Parts};
use forklaunch_axum::native::{EndpointError, EndpointPolicy, PolicyCapabilities, RouteContract};
use forklaunch_core::contract::bindings::AuthorizationPolicy;
use uuid::Uuid;

pub const COOKIE_NAME: &str = "loandesk_session";
pub const RELATIONSHIP_MANAGER: &str = "relationship_manager";
pub const CREDIT_OFFICER: &str = "credit_officer";
pub const ADMIN: &str = "admin";
pub const ROLES: [&str; 3] = [RELATIONSHIP_MANAGER, CREDIT_OFFICER, ADMIN];

/// The signed-in user, as of this request.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub mfa_enabled: bool,
    /// SHA-256 of the session token, to end exactly this session on sign-out.
    pub session_hash: String,
}
impl CurrentUser {
    pub fn to_session_user(&self) -> SessionUser {
        SessionUser {
            id: self.id.to_string(),
            email: self.email.clone(),
            full_name: self.full_name.clone(),
            role: self.role.clone(),
            mfa_enabled: self.mfa_enabled,
        }
    }
    pub fn is(&self, role: &str) -> bool {
        self.role == role
    }
}

pub fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty())
}

fn denial_message(role: &str, allowed: &[String]) -> &'static str {
    if allowed.iter().any(|allowed| allowed == ADMIN) {
        "Only admins can manage users"
    } else if role == ADMIN {
        "Admins manage users and do not have access to applications"
    } else if allowed.len() == 1 && allowed[0] == RELATIONSHIP_MANAGER {
        "Only relationship managers can create applications and upload documents"
    } else if allowed.len() == 1 && allowed[0] == CREDIT_OFFICER {
        "Only credit officers can approve or decline applications"
    } else {
        "You do not have access to this"
    }
}

#[derive(Clone, Copy)]
pub struct SessionCookiePolicy;

impl AuthorizationPolicy for SessionCookiePolicy {}

impl EndpointPolicy<AppState> for SessionCookiePolicy {
    const CAPABILITIES: PolicyCapabilities = PolicyCapabilities::Authorized;
    type Context = CurrentUser;

    async fn authorize(
        &self,
        parts: &mut Parts,
        state: &AppState,
        route: &RouteContract,
    ) -> Result<CurrentUser, EndpointError> {
        let not_signed_in = || AppError::Unauthorized("Not signed in".into()).into_endpoint();
        let Some(token) = session_cookie(&parts.headers) else {
            return Err(not_signed_in());
        };
        let user = match state.auth.resolve_session(token).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::warn!(
                    event = "invalid_session",
                    operation = %route.operation_name,
                    request_id = %transport::request_id(),
                    "session cookie rejected: unknown, expired or revoked"
                );
                return Err(not_signed_in());
            }
            Err(error) => return Err(error.into_endpoint()),
        };
        transport::set_user(user.id);

        // Per-user limits, before any work is done for the request.
        let caller = user.id.to_string();
        let expensive = EXPENSIVE_OPERATIONS.contains(&route.name.as_str());
        let refused = if !state.limits.requests.allow(&caller) {
            Some("requests")
        } else if expensive && !state.limits.expensive.allow(&caller) {
            Some("expensive")
        } else {
            None
        };
        if let Some(limit) = refused {
            tracing::warn!(
                event = "rate_limited",
                limit,
                user_id = %user.id,
                operation = %route.operation_name,
                request_id = %transport::request_id(),
                "request refused: rate limit reached"
            );
            return Err(AppError::TooManyRequests(
                "Too many requests. Wait a moment and try again.".into(),
            )
            .into_endpoint());
        }

        let allowed = route.rbac.as_ref().map(|rbac| rbac.allowed_roles.as_slice()).unwrap_or(&[]);
        if !allowed.is_empty() && !allowed.contains(&user.role) {
            state.auth.log_denial(&user, &route.operation_name, None, "role not allowed");
            return Err(
                AppError::Forbidden(denial_message(&user.role, allowed).into()).into_endpoint()
            );
        }
        if allowed.iter().any(|role| role == ADMIN)
            && state.settings.mfa_required_for_admins
            && !user.mfa_enabled
        {
            state.auth.log_denial(&user, &route.operation_name, None, "second factor not enrolled");
            return Err(AppError::Forbidden(
                "Turn on two-factor sign-in before managing users".into(),
            )
            .into_endpoint());
        }
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn finds_the_session_cookie_among_others_and_ignores_empty_values() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_static("a=1; loandesk_session=abc; b=2"));
        assert_eq!(session_cookie(&headers), Some("abc"));
        headers.insert(header::COOKIE, HeaderValue::from_static("loandesk_session="));
        assert_eq!(session_cookie(&headers), None);
    }

    #[test]
    fn denial_messages_match_the_role_rules() {
        let lending = vec![RELATIONSHIP_MANAGER.to_owned(), CREDIT_OFFICER.to_owned()];
        assert_eq!(
            denial_message(ADMIN, &lending),
            "Admins manage users and do not have access to applications"
        );
        assert_eq!(
            denial_message(RELATIONSHIP_MANAGER, &[CREDIT_OFFICER.to_owned()]),
            "Only credit officers can approve or decline applications"
        );
        assert_eq!(
            denial_message(CREDIT_OFFICER, &[ADMIN.to_owned()]),
            "Only admins can manage users"
        );
    }
}
