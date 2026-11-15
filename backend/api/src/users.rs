use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{audit, CurrentUser, Role},
    crypto,
    error::{bad, ApiResult, AppError},
    AppState,
};

const ADMIN_ONLY: &str = "Only admins can manage users";
pub const MIN_PASSWORD_LEN: usize = 12;

#[derive(Serialize, sqlx::FromRow)]
pub struct UserRow {
    id: Uuid,
    email: String,
    full_name: String,
    role: String,
    created_at: DateTime<Utc>,
}

pub async fn list(State(st): State<AppState>, user: CurrentUser) -> ApiResult<Json<Vec<UserRow>>> {
    user.require_role(Role::Admin, ADMIN_ONLY)?;
    let rows = sqlx::query_as("SELECT id, email, full_name, role, created_at FROM users ORDER BY full_name")
        .fetch_all(&st.db)
        .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct NewUser {
    email: String,
    full_name: String,
    role: Role,
    password: String,
}

pub async fn create(
    State(st): State<AppState>,
    user: CurrentUser,
    Json(body): Json<NewUser>,
) -> ApiResult<(StatusCode, Json<UserRow>)> {
    user.require_role(Role::Admin, ADMIN_ONLY)?;

    let email = body.email.trim().to_lowercase();
    let full_name = body.full_name.trim().to_string();
    let valid_email = email.len() <= 254
        && !email.contains(char::is_whitespace)
        && email
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
    if !valid_email {
        return Err(bad("Enter a valid email address"));
    }
    if full_name.is_empty() || full_name.chars().count() > 200 {
        return Err(bad("Full name is required"));
    }
    if body.password.chars().count() < MIN_PASSWORD_LEN || body.password.len() > 1024 {
        return Err(bad(format!(
            "Password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }

    let password = body.password;
    let hash = tokio::task::spawn_blocking(move || crypto::hash_password(&password)).await??;

    let row: Option<UserRow> = sqlx::query_as(
        "INSERT INTO users (id, email, full_name, role, password_hash)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (email) DO NOTHING
         RETURNING id, email, full_name, role, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&full_name)
    .bind(body.role.as_str())
    .bind(hash)
    .fetch_optional(&st.db)
    .await?;
    let row = row.ok_or(AppError::Conflict("A user with that email already exists"))?;
    audit(&st.db, Some(user.id), "user_created", Some(row.id), Some(body.role.as_str())).await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Deserialize)]
pub struct SetRole {
    role: Role,
}

pub async fn set_role(
    State(st): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SetRole>,
) -> ApiResult<Json<UserRow>> {
    user.require_role(Role::Admin, ADMIN_ONLY)?;
    if id == user.id {
        // Stops an admin from locking the team out by demoting the last
        // admin, and from handing themselves another role.
        return Err(AppError::Forbidden("You cannot change your own role"));
    }
    let row: UserRow = sqlx::query_as(
        "UPDATE users SET role = $1 WHERE id = $2
         RETURNING id, email, full_name, role, created_at",
    )
    .bind(body.role.as_str())
    .bind(id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    audit(&st.db, Some(user.id), "user_role_changed", Some(id), Some(body.role.as_str())).await?;
    Ok(Json(row))
}
