//! User administration (admins only; enforced by the route contracts).
use crate::{
    api::policy::{CurrentUser, ROLES},
    domain::{
        crypto,
        schemas::{
            PageQuery,
            users::{NewUser, User},
        },
    },
    error::{AppError, Result, bad, storage},
    persistence::Db,
};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

pub const MIN_PASSWORD_LEN: usize = 12;
const DEFAULT_PAGE: i64 = 50;

#[derive(Clone)]
pub struct UsersService {
    db: Db,
}

fn user(row: &PgRow) -> User {
    User {
        id: row.get::<Uuid, _>("id").to_string(),
        email: row.get("email"),
        full_name: row.get("full_name"),
        role: row.get("role"),
        created_at: super::applications::wire_time(row.get("created_at")),
    }
}

fn role(value: &str) -> Result<&str> {
    ROLES.iter().find(|role| **role == value).copied().ok_or_else(|| bad("Choose a valid role"))
}

/// Cursor for (full_name, id) ordering: hex of the name, then the id.
fn encode_cursor(name: &str, id: &str) -> String {
    format!("{}.{id}", hex::encode(name.as_bytes()))
}
fn decode_cursor(cursor: &str) -> Result<(String, Uuid)> {
    let invalid = || bad("That page link is no longer valid");
    let (name, id) = cursor.split_once('.').ok_or_else(invalid)?;
    let name =
        String::from_utf8(hex::decode(name).map_err(|_| invalid())?).map_err(|_| invalid())?;
    Ok((name, Uuid::parse_str(id).map_err(|_| invalid())?))
}

impl UsersService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// One page ordered by (full_name, id): a unique, indexed key, so pages
    /// neither skip nor repeat rows and deep pages cost the same as the first.
    pub async fn list(&self, query: PageQuery) -> Result<(Vec<User>, Option<String>)> {
        let limit = query.limit.into_option().unwrap_or(DEFAULT_PAGE).clamp(1, 100);
        let after = query.cursor.into_option().map(|cursor| decode_cursor(&cursor)).transpose()?;
        let (after_name, after_id) = after.unzip();
        let mut tx = self.db.tx().await?;
        let rows = sqlx::query(
            "SELECT id, email, full_name, role, created_at FROM users
             WHERE ($1::text IS NULL OR (full_name, id) > ($1, $2))
             ORDER BY full_name, id LIMIT $3",
        )
        .bind(after_name)
        .bind(after_id)
        .bind(limit + 1)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("list users"))?;
        tx.commit().await.map_err(storage("list users"))?;
        let mut users: Vec<User> = rows.iter().map(user).collect();
        let next = if users.len() > usize::try_from(limit).unwrap_or(usize::MAX) {
            users.pop();
            users.last().map(|last| encode_cursor(&last.full_name, &last.id))
        } else {
            None
        };
        Ok((users, next))
    }

    pub async fn create(&self, admin: &CurrentUser, body: NewUser) -> Result<User> {
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
        let role = role(&body.role)?;
        if body.password.chars().count() < MIN_PASSWORD_LEN || body.password.len() > 1024 {
            return Err(bad(format!("Password must be at least {MIN_PASSWORD_LEN} characters")));
        }
        let password = body.password;
        let hash = tokio::task::spawn_blocking(move || crypto::hash_password(&password))
            .await
            .map_err(storage("hash password"))?
            .map_err(storage("hash password"))?;

        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "INSERT INTO users (id, email, full_name, role, password_hash)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (email) DO NOTHING
             RETURNING id, email, full_name, role, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(&full_name)
        .bind(role)
        .bind(hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("create user"))?
        .ok_or_else(|| AppError::Conflict("A user with that email already exists".into()))?;
        let created = user(&row);
        self.db
            .audit(&mut tx, Some(admin.id), "user_created", Some(row.get("id")), Some(role))
            .await?;
        tx.commit().await.map_err(storage("create user"))?;
        Ok(created)
    }

    pub async fn set_role(&self, admin: &CurrentUser, id: &str, new_role: &str) -> Result<User> {
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        let new_role = role(new_role)?;
        if id == admin.id {
            // Stops an admin from locking the team out by demoting the last
            // admin, and from handing themselves another role.
            return Err(AppError::Forbidden("You cannot change your own role".into()));
        }
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "UPDATE users SET role = $1 WHERE id = $2
             RETURNING id, email, full_name, role, created_at",
        )
        .bind(new_role)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("change role"))?
        .ok_or(AppError::NotFound)?;
        // Access is revoked immediately: the role is read on every request,
        // and the user's open sessions are ended so they sign in again.
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(storage("end sessions"))?;
        self.db
            .audit(&mut tx, Some(admin.id), "user_role_changed", Some(id), Some(new_role))
            .await?;
        tx.commit().await.map_err(storage("change role"))?;
        Ok(user(&row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_round_trip_and_reject_tampering() {
        let id = Uuid::new_v4();
        let cursor = encode_cursor("Maya Chen", &id.to_string());
        assert_eq!(decode_cursor(&cursor).unwrap(), ("Maya Chen".to_owned(), id));
        assert!(decode_cursor("not-a-cursor").is_err());
        assert!(decode_cursor("zz.00000000-0000-0000-0000-000000000000").is_err());
    }
}
