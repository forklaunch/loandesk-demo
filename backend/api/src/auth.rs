use axum::{
    extract::{FromRequestParts, State},
    http::{header, request::Parts, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    crypto,
    error::{bad, ApiResult, AppError},
    AppState,
};

const COOKIE_NAME: &str = "loandesk_session";
const SESSION_HOURS: i64 = 8;
const MAX_FAILED_LOGINS: i32 = 5;
const LOCKOUT_MINUTES: i32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    RelationshipManager,
    CreditOfficer,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::RelationshipManager => "relationship_manager",
            Role::CreditOfficer => "credit_officer",
            Role::Admin => "admin",
        }
    }
}

impl TryFrom<String> for Role {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        match s.as_str() {
            "relationship_manager" => Ok(Role::RelationshipManager),
            "credit_officer" => Ok(Role::CreditOfficer),
            "admin" => Ok(Role::Admin),
            other => Err(format!("unknown role {other}")),
        }
    }
}

/// The logged-in user. The role is read from the database on every request,
/// so a role change takes effect immediately.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    #[sqlx(try_from = "String")]
    pub role: Role,
}

impl CurrentUser {
    pub fn require_role(&self, role: Role, msg: &'static str) -> ApiResult<()> {
        if self.role == role {
            Ok(())
        } else {
            Err(AppError::Forbidden(msg))
        }
    }
}

fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value)
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token =
            session_cookie(&parts.headers).ok_or(AppError::Unauthorized("Not signed in"))?;
        sqlx::query_as::<_, CurrentUser>(
            "SELECT u.id, u.email, u.full_name, u.role
             FROM sessions s JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = $1 AND s.expires_at > now()",
        )
        .bind(crypto::sha256_hex(token))
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized("Not signed in"))
    }
}

pub async fn audit(
    db: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    entity_id: Option<Uuid>,
    detail: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO audit_log (user_id, action, entity_id, detail) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(action)
        .bind(entity_id)
        .bind(detail)
        .execute(db)
        .await?;
    Ok(())
}

fn cookie_header(value: &str, max_age: i64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}{secure}"
    ))
    .expect("cookie is ascii")
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(State(st): State<AppState>, Json(body): Json<LoginRequest>) -> ApiResult<Response> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || email.len() > 254 || body.password.is_empty() || body.password.len() > 1024 {
        return Err(bad("Email and password are required"));
    }

    let found: Option<(Uuid, String, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT id, password_hash, locked_until FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(&st.db)
            .await?;

    if let Some((_, _, Some(until))) = &found {
        if *until > Utc::now() {
            return Err(AppError::TooManyRequests(
                "Too many failed sign-in attempts. Try again in a few minutes.",
            ));
        }
    }

    // Always run a hash verification so response time does not reveal
    // whether the email exists.
    let hash = found
        .as_ref()
        .map(|(_, h, _)| h.clone())
        .unwrap_or_else(|| st.dummy_hash.as_ref().clone());
    let password = body.password;
    let ok = tokio::task::spawn_blocking(move || crypto::verify_password(&password, &hash)).await?;

    let user_id = match (found, ok) {
        (Some((id, _, _)), true) => id,
        (found, _) => {
            if let Some((id, _, _)) = found {
                sqlx::query(
                    "UPDATE users SET failed_logins = failed_logins + 1,
                        locked_until = CASE WHEN failed_logins + 1 >= $2
                            THEN now() + make_interval(mins => $3) ELSE locked_until END
                     WHERE id = $1",
                )
                .bind(id)
                .bind(MAX_FAILED_LOGINS)
                .bind(LOCKOUT_MINUTES)
                .execute(&st.db)
                .await?;
                audit(&st.db, Some(id), "login_failed", None, None).await?;
            }
            return Err(AppError::Unauthorized("Invalid email or password"));
        }
    };

    sqlx::query("UPDATE users SET failed_logins = 0, locked_until = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&st.db)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
        .execute(&st.db)
        .await?;

    let token = crypto::new_session_token();
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, expires_at)
         VALUES ($1, $2, now() + make_interval(hours => $3))",
    )
    .bind(crypto::sha256_hex(&token))
    .bind(user_id)
    .bind(SESSION_HOURS as i32)
    .execute(&st.db)
    .await?;
    audit(&st.db, Some(user_id), "login", None, None).await?;

    let user: CurrentUser =
        sqlx::query_as("SELECT id, email, full_name, role FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&st.db)
            .await?;

    let mut res = Json(user).into_response();
    res.headers_mut().insert(
        header::SET_COOKIE,
        cookie_header(&token, SESSION_HOURS * 3600, st.cookie_secure),
    );
    Ok(res)
}

pub async fn logout(State(st): State<AppState>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(token) = session_cookie(&headers) {
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(crypto::sha256_hex(token))
            .execute(&st.db)
            .await?;
    }
    let mut res = Json(json!({ "ok": true })).into_response();
    res.headers_mut()
        .insert(header::SET_COOKIE, cookie_header("", 0, st.cookie_secure));
    Ok(res)
}

pub async fn me(user: CurrentUser) -> Json<CurrentUser> {
    Json(user)
}
