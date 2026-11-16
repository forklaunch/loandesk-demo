//! User administration (admins only; enforced by the route contracts).
use crate::{
    api::{
        policy::{CurrentUser, ROLES},
        transport,
    },
    config::Settings,
    domain::{
        crypto,
        schemas::{
            PageQuery,
            users::{
                AccessReview, AccessReviewEntry, AccessReviewRecorded, AuditEntry, NewUser, User,
            },
        },
    },
    error::{AppError, Result, bad, storage},
    persistence::Db,
};
use chrono::{DateTime, Utc};
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

pub const MIN_PASSWORD_LEN: usize = 12;
const DEFAULT_PAGE: i64 = 50;
/// Accounts in one access review. A lending team is far below this; a larger
/// directory gets `truncated: true` rather than an unbounded response.
const ACCESS_REVIEW_LIMIT: i64 = 1000;
/// Audit action of a recorded access review; the newest one is "the last review".
pub const ACCESS_REVIEW_RECORDED: &str = "access_review_recorded";

#[derive(Clone)]
pub struct UsersService {
    db: Db,
    settings: Settings,
}

/// Whole days an account has gone unused: since its last sign-in, or since
/// its baseline (creation or re-enabling) when that is more recent.
pub fn days_unused(
    now: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
    baseline: DateTime<Utc>,
) -> i64 {
    let last_use = last_login.map_or(baseline, |login| login.max(baseline));
    (now - last_use).num_days().max(0)
}

/// A review is overdue when the last one is older than the interval, or, if
/// none was ever recorded, when accounts have existed for longer than that.
pub fn review_overdue(
    now: DateTime<Utc>,
    last_review: Option<DateTime<Utc>>,
    oldest_account: Option<DateTime<Utc>>,
    interval_days: i64,
) -> bool {
    last_review.or(oldest_account).is_some_and(|since| (now - since).num_days() > interval_days)
}

/// An active account is stale once it has been unused for more than `stale_after_days`.
pub fn is_stale(disabled: bool, days_unused: i64, stale_after_days: i64) -> bool {
    !disabled && days_unused > stale_after_days
}

fn user(row: &PgRow) -> User {
    User {
        id: row.get::<Uuid, _>("id").to_string(),
        email: row.get("email"),
        full_name: row.get("full_name"),
        role: row.get("role"),
        created_at: super::applications::wire_time(row.get("created_at")),
        last_login_at: row
            .get::<Option<DateTime<Utc>>, _>("last_login_at")
            .map(super::applications::wire_time),
        disabled_at: row
            .get::<Option<DateTime<Utc>>, _>("disabled_at")
            .map(super::applications::wire_time),
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
    pub fn new(db: Db, settings: Settings) -> Self {
        Self { db, settings }
    }

    /// One page ordered by (full_name, id): a unique, indexed key, so pages
    /// neither skip nor repeat rows and deep pages cost the same as the first.
    pub async fn list(&self, query: PageQuery) -> Result<(Vec<User>, Option<String>)> {
        let limit = query.limit.into_option().unwrap_or(DEFAULT_PAGE).clamp(1, 100);
        let after = query.cursor.into_option().map(|cursor| decode_cursor(&cursor)).transpose()?;
        let (after_name, after_id) = after.unzip();
        let mut tx = self.db.tx().await?;
        let rows = sqlx::query(
            "SELECT id, email, full_name, role, created_at, last_login_at, disabled_at FROM users
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
             RETURNING id, email, full_name, role, created_at, last_login_at, disabled_at",
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
        tracing::info!(
            event = "user_created",
            user_id = %admin.id,
            account = %created.id,
            role,
            request_id = %transport::request_id(),
            "user account created"
        );
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
             RETURNING id, email, full_name, role, created_at, last_login_at, disabled_at",
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
        tracing::info!(
            event = "user_role_changed",
            user_id = %admin.id,
            account = %id,
            role = new_role,
            request_id = %transport::request_id(),
            "user role changed; that user's sessions ended"
        );
        Ok(user(&row))
    }

    /// One page of the audit trail, newest first, by its primary key: deep
    /// pages cost the same as the first. Admins only (route contract).
    pub async fn audit_log(&self, query: PageQuery) -> Result<(Vec<AuditEntry>, Option<String>)> {
        let limit = query.limit.into_option().unwrap_or(DEFAULT_PAGE).clamp(1, 100);
        let before = query
            .cursor
            .into_option()
            .map(|cursor| {
                cursor.parse::<i64>().map_err(|_| bad("That page link is no longer valid"))
            })
            .transpose()?;
        let mut tx = self.db.tx().await?;
        let rows = sqlx::query(
            "SELECT id, at, user_id, action, entity_id, detail, request_id FROM audit_log
             WHERE ($1::bigint IS NULL OR id < $1) ORDER BY id DESC LIMIT $2",
        )
        .bind(before)
        .bind(limit + 1)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("list audit trail"))?;
        tx.commit().await.map_err(storage("list audit trail"))?;
        let mut entries: Vec<AuditEntry> = rows
            .iter()
            .map(|row| AuditEntry {
                id: row.get("id"),
                at: super::applications::wire_time(row.get("at")),
                user_id: row.get::<Option<Uuid>, _>("user_id").map(|id| id.to_string()),
                action: row.get("action"),
                entity_id: row.get::<Option<Uuid>, _>("entity_id").map(|id| id.to_string()),
                detail: row.get("detail"),
                request_id: row.get("request_id"),
            })
            .collect();
        let next = if entries.len() > usize::try_from(limit).unwrap_or(usize::MAX) {
            entries.pop();
            entries.last().map(|last| last.id.to_string())
        } else {
            None
        };
        Ok((entries, next))
    }

    /// Disables or re-enables another user's account. A disabled account
    /// cannot sign in and its open sessions end in the same transaction, so
    /// access is revoked at once. Re-enabling restarts the inactivity count.
    pub async fn set_disabled(
        &self,
        admin: &CurrentUser,
        id: &str,
        disabled: bool,
    ) -> Result<User> {
        let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
        if id == admin.id {
            return Err(AppError::Forbidden("You cannot disable your own account".into()));
        }
        let mut tx = self.db.tx().await?;
        let row = if disabled {
            sqlx::query(
                "UPDATE users SET disabled_at = COALESCE(disabled_at, now()),
                        disabled_reason = COALESCE(disabled_reason, 'admin')
                 WHERE id = $1
                 RETURNING id, email, full_name, role, created_at, last_login_at, disabled_at",
            )
        } else {
            sqlx::query(
                "UPDATE users SET disabled_at = NULL, disabled_reason = NULL,
                        access_baseline_at = now(), failed_logins = 0, locked_until = NULL
                 WHERE id = $1
                 RETURNING id, email, full_name, role, created_at, last_login_at, disabled_at",
            )
        }
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("change account status"))?
        .ok_or(AppError::NotFound)?;
        let sessions_ended = if disabled {
            sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(storage("end sessions"))?
                .rows_affected()
        } else {
            0
        };
        let action = if disabled { "user_disabled" } else { "user_enabled" };
        self.db.audit(&mut tx, Some(admin.id), action, Some(id), Some("admin")).await?;
        tx.commit().await.map_err(storage("change account status"))?;
        tracing::info!(
            event = action,
            user_id = %admin.id,
            account = %id,
            sessions_ended,
            request_id = %transport::request_id(),
            "account status changed by an admin"
        );
        Ok(user(&row))
    }

    /// The access review: every account with its role, second factor, last
    /// sign-in, open sessions and whether it is stale, plus when a review was
    /// last recorded. Bounded: at most `ACCESS_REVIEW_LIMIT` accounts.
    pub async fn access_review(&self) -> Result<AccessReview> {
        let mut tx = self.db.tx().await?;
        let (review, _) = self.build_access_review(&mut tx).await?;
        tx.commit().await.map_err(storage("build access review"))?;
        Ok(review)
    }

    async fn build_access_review(
        &self,
        tx: &mut crate::persistence::Tx<'_>,
    ) -> Result<(AccessReview, i64)> {
        let rows = sqlx::query(
            "SELECT u.id, u.email, u.full_name, u.role, u.created_at, u.last_login_at,
                    u.access_baseline_at, u.password_changed_at, u.disabled_at, u.disabled_reason,
                    EXISTS (SELECT 1 FROM user_mfa m
                             WHERE m.user_id = u.id AND m.confirmed_at IS NOT NULL) AS mfa_enabled,
                    (SELECT count(*) FROM sessions s
                      WHERE s.user_id = u.id AND s.expires_at > now()) AS open_sessions
             FROM users u ORDER BY u.full_name, u.id LIMIT $1",
        )
        .bind(ACCESS_REVIEW_LIMIT + 1)
        .fetch_all(&mut **tx)
        .await
        .map_err(storage("build access review"))?;
        let last = sqlx::query(
            "SELECT at, user_id FROM audit_log WHERE action = $1 ORDER BY at DESC LIMIT 1",
        )
        .bind(ACCESS_REVIEW_RECORDED)
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage("find last access review"))?;

        let now = Utc::now();
        let stale_after_days = self.settings.access_stale_days;
        let truncated = rows.len() > usize::try_from(ACCESS_REVIEW_LIMIT).unwrap_or(usize::MAX);
        let users: Vec<AccessReviewEntry> = rows
            .iter()
            .take(usize::try_from(ACCESS_REVIEW_LIMIT).unwrap_or(usize::MAX))
            .map(|row| {
                let disabled_at: Option<DateTime<Utc>> = row.get("disabled_at");
                let last_login_at: Option<DateTime<Utc>> = row.get("last_login_at");
                let days = days_unused(now, last_login_at, row.get("access_baseline_at"));
                AccessReviewEntry {
                    id: row.get::<Uuid, _>("id").to_string(),
                    email: row.get("email"),
                    full_name: row.get("full_name"),
                    role: row.get("role"),
                    created_at: super::applications::wire_time(row.get("created_at")),
                    last_login_at: last_login_at.map(super::applications::wire_time),
                    password_changed_at: row
                        .get::<Option<DateTime<Utc>>, _>("password_changed_at")
                        .map(super::applications::wire_time),
                    mfa_enabled: row.get("mfa_enabled"),
                    status: if disabled_at.is_some() { "disabled" } else { "active" }.to_owned(),
                    disabled_at: disabled_at.map(super::applications::wire_time),
                    disabled_reason: row.get("disabled_reason"),
                    days_since_last_use: days,
                    stale: is_stale(disabled_at.is_some(), days, stale_after_days),
                    open_sessions: row.get("open_sessions"),
                }
            })
            .collect();
        let last_review_at: Option<DateTime<Utc>> = last.as_ref().map(|row| row.get("at"));
        let oldest_account: Option<DateTime<Utc>> =
            rows.iter().map(|row| row.get::<DateTime<Utc>, _>("created_at")).min();
        let review_interval_days = self.settings.access_review_interval_days;
        let disabled = users.iter().filter(|user| user.status == "disabled").count();
        let review = AccessReview {
            generated_at: super::applications::wire_time(now),
            stale_after_days,
            review_interval_days,
            last_review_at: last_review_at.map(super::applications::wire_time),
            last_review_by: last
                .as_ref()
                .and_then(|row| row.get::<Option<Uuid>, _>("user_id"))
                .map(|id| id.to_string()),
            review_overdue: review_overdue(
                now,
                last_review_at,
                oldest_account,
                review_interval_days,
            ),
            stale_accounts: i64::try_from(users.iter().filter(|user| user.stale).count())
                .unwrap_or(i64::MAX),
            truncated,
            users,
        };
        Ok((review, i64::try_from(disabled).unwrap_or(i64::MAX)))
    }

    /// Records that `admin` reviewed the accounts as they are now. The entry
    /// in the append-only audit trail (who, when, how many accounts, how many
    /// stale and disabled) is the evidence that the review took place; it
    /// cannot be edited or back-dated.
    pub async fn record_access_review(&self, admin: &CurrentUser) -> Result<AccessReviewRecorded> {
        let mut tx = self.db.tx().await?;
        let (review, disabled) = self.build_access_review(&mut tx).await?;
        let accounts = i64::try_from(review.users.len()).unwrap_or(i64::MAX);
        let detail =
            format!("accounts={accounts} stale={} disabled={disabled}", review.stale_accounts);
        self.db.audit(&mut tx, Some(admin.id), ACCESS_REVIEW_RECORDED, None, Some(&detail)).await?;
        tx.commit().await.map_err(storage("record access review"))?;
        tracing::info!(
            event = "access_review_recorded",
            user_id = %admin.id,
            accounts,
            stale_accounts = review.stale_accounts,
            disabled_accounts = disabled,
            request_id = %transport::request_id(),
            "access review recorded"
        );
        Ok(AccessReviewRecorded {
            reviewed_at: review.generated_at,
            accounts,
            stale_accounts: review.stale_accounts,
            disabled_accounts: disabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_review_flags_accounts_unused_for_longer_than_the_stale_period() {
        let now = Utc::now();
        let days = chrono::Duration::days;
        // Never signed in: counted from the baseline (creation).
        assert_eq!(days_unused(now, None, now - days(120)), 120);
        // Signed in since: counted from the sign-in.
        assert_eq!(days_unused(now, Some(now - days(3)), now - days(400)), 3);
        // Re-enabled after a long absence: counted from re-enabling, not the old sign-in.
        assert_eq!(days_unused(now, Some(now - days(400)), now - days(1)), 1);
        assert!(is_stale(false, 91, 90));
        assert!(!is_stale(false, 90, 90), "exactly the period is not yet stale");
        assert!(!is_stale(true, 500, 90), "a disabled account is already revoked");
        // The review itself is due every interval, counted from the last
        // recorded one, or from the first account when none was recorded.
        assert!(!review_overdue(now, Some(now - days(90)), Some(now - days(900)), 90));
        assert!(review_overdue(now, Some(now - days(91)), Some(now - days(900)), 90));
        assert!(review_overdue(now, None, Some(now - days(91)), 90));
        assert!(!review_overdue(now, None, Some(now - days(5)), 90));
        assert!(!review_overdue(now, None, None, 90), "no accounts yet");
    }

    #[test]
    fn cursors_round_trip_and_reject_tampering() {
        let id = Uuid::new_v4();
        let cursor = encode_cursor("Maya Chen", &id.to_string());
        assert_eq!(decode_cursor(&cursor).unwrap(), ("Maya Chen".to_owned(), id));
        assert!(decode_cursor("not-a-cursor").is_err());
        assert!(decode_cursor("zz.00000000-0000-0000-0000-000000000000").is_err());
    }
}
