//! Sign-in, sessions and two-factor enrolment.
use crate::{
    api::{
        policy::{COOKIE_NAME, CurrentUser},
        transport,
    },
    config::Settings,
    domain::{
        crypto,
        schemas::auth::{LoginRequest, MfaEnrolment, Ok as OkBody, SessionUser},
        totp,
    },
    error::{AppError, Result, bad, storage},
    persistence::{Db, Disclosures},
};
use chrono::{DateTime, Utc};
use rand::RngCore;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

const MAX_FAILED_LOGINS: i32 = 5;
const LOCKOUT_MINUTES: i32 = 15;
/// One answer for a wrong password, an unknown email and a wrong or missing
/// two-factor code, so the response never says which accounts exist.
const INVALID_LOGIN: &str = "Invalid email or password";

#[derive(Clone)]
pub struct AuthService {
    db: Db,
    settings: Settings,
    dummy_hash: Arc<String>,
}

fn unix_now() -> u64 {
    u64::try_from(Utc::now().timestamp()).unwrap_or(0)
}

impl AuthService {
    pub fn new(db: Db, settings: Settings) -> Result<Self> {
        let dummy_hash = crypto::hash_password("not-a-real-password")
            .map_err(storage("prepare password hasher"))?;
        Ok(Self { db, settings, dummy_hash: Arc::new(dummy_hash) })
    }

    fn cookie(&self, value: &str, max_age: i64) -> String {
        let secure = if self.settings.cookie_secure { "; Secure" } else { "" };
        format!(
            "{COOKIE_NAME}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}{secure}"
        )
    }

    /// Named, structured record of a refused action: who, what, on which record.
    pub fn log_denial(
        &self,
        user: &CurrentUser,
        action: &str,
        resource: Option<Uuid>,
        reason: &str,
    ) {
        tracing::warn!(
            event = "permission_denied",
            user_id = %user.id,
            role = %user.role,
            action,
            resource = %resource.map_or_else(|| "-".to_owned(), |id| id.to_string()),
            reason,
            request_id = %transport::request_id(),
            "permission denied"
        );
    }

    pub async fn resolve_session(&self, token: &str) -> Result<Option<CurrentUser>> {
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok(None);
        }
        let hash = crypto::sha256_hex(token);
        let mut tx = self.db.tx().await?;
        let row = sqlx::query(
            "SELECT u.id, u.email, u.full_name, u.role,
                    EXISTS (SELECT 1 FROM user_mfa m WHERE m.user_id = u.id AND m.confirmed_at IS NOT NULL) AS mfa_enabled
             FROM sessions s JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = $1 AND s.expires_at > now()",
        )
        .bind(&hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("resolve session"))?;
        tx.commit().await.map_err(storage("resolve session"))?;
        Ok(row.map(|row| CurrentUser {
            id: row.get("id"),
            email: row.get("email"),
            full_name: row.get("full_name"),
            role: row.get("role"),
            mfa_enabled: row.get("mfa_enabled"),
            session_hash: hash,
        }))
    }

    pub async fn login(&self, input: LoginRequest) -> Result<SessionUser> {
        let email = input.email.trim().to_lowercase();
        if email.is_empty()
            || email.len() > 254
            || input.password.is_empty()
            || input.password.len() > 1024
        {
            return Err(bad("Email and password are required"));
        }
        let mut tx = self.db.tx().await?;
        let found = sqlx::query(
            "SELECT u.id, u.full_name, u.role, u.password_hash, u.locked_until,
                    (SELECT m.secret_ciphertext FROM user_mfa m
                      WHERE m.user_id = u.id AND m.confirmed_at IS NOT NULL) AS mfa_secret
             FROM users u WHERE u.email = $1 FOR UPDATE",
        )
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("look up user"))?;

        if let Some(row) = &found {
            let locked_until: Option<DateTime<Utc>> = row.get("locked_until");
            if locked_until.is_some_and(|until| until > Utc::now()) {
                let id: Uuid = row.get("id");
                tracing::warn!(
                    event = "login_rate_limited",
                    user_id = %id,
                    request_id = %transport::request_id(),
                    "sign-in refused: account is temporarily locked"
                );
                return Err(AppError::TooManyRequests(
                    "Too many failed sign-in attempts. Try again in a few minutes.".into(),
                ));
            }
        }

        // Always run a hash verification so response time does not reveal
        // whether the email exists.
        let hash = found.as_ref().map_or_else(
            || self.dummy_hash.as_ref().clone(),
            |row| row.get::<String, _>("password_hash"),
        );
        let password = input.password;
        let mut ok = tokio::task::spawn_blocking(move || crypto::verify_password(&password, &hash))
            .await
            .map_err(storage("verify password"))?;

        let disclosures = Disclosures::new();
        let mut mfa_enabled = false;
        if let (true, Some(row)) = (ok, &found) {
            let id: Uuid = row.get("id");
            if let Some(ciphertext) = row.get::<Option<String>, _>("mfa_secret") {
                mfa_enabled = true;
                let secret = self
                    .db
                    .reveal(
                        &self.db.members.mfa_secret,
                        id,
                        ciphertext,
                        id,
                        "verify-sign-in-code",
                        &disclosures,
                    )
                    .await?;
                let code = input.totp.into_option().unwrap_or_default();
                ok = totp::verify(&secret, &code, unix_now());
            }
        }

        let Some(row) = found.filter(|_| ok) else {
            // Unknown email: nothing to count. Known email: count and maybe lock.
            let user_id: Option<Uuid> = sqlx::query_scalar(
                "UPDATE users SET failed_logins = failed_logins + 1,
                        locked_until = CASE WHEN failed_logins + 1 >= $2
                            THEN now() + make_interval(mins => $3) ELSE locked_until END
                 WHERE email = $1 RETURNING id",
            )
            .bind(&email)
            .bind(MAX_FAILED_LOGINS)
            .bind(LOCKOUT_MINUTES)
            .fetch_optional(&mut *tx)
            .await
            .map_err(storage("record failed sign-in"))?;
            if user_id.is_some() {
                self.db.audit(&mut tx, user_id, "login_failed", None, None).await?;
            }
            drop(disclosures);
            tx.commit().await.map_err(storage("record failed sign-in"))?;
            tracing::warn!(
                event = "login_failed",
                user_id = %user_id.map_or_else(|| "unknown".to_owned(), |id| id.to_string()),
                request_id = %transport::request_id(),
                "failed sign-in"
            );
            return Err(AppError::Unauthorized(INVALID_LOGIN.into()));
        };

        let user_id: Uuid = row.get("id");
        sqlx::query("UPDATE users SET failed_logins = 0, locked_until = NULL WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(storage("reset failed sign-ins"))?;
        sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
            .execute(&mut *tx)
            .await
            .map_err(storage("remove expired sessions"))?;
        let token = crypto::new_session_token();
        sqlx::query(
            "INSERT INTO sessions (token_hash, user_id, expires_at)
             VALUES ($1, $2, now() + make_interval(hours => $3))",
        )
        .bind(crypto::sha256_hex(&token))
        .bind(user_id)
        .bind(self.settings.session_hours)
        .execute(&mut *tx)
        .await
        .map_err(storage("create session"))?;
        self.db.audit(&mut tx, Some(user_id), "login", None, None).await?;
        disclosures.persist(&mut tx, user_id).await?;
        tx.commit().await.map_err(storage("create session"))?;

        transport::set_user(user_id);
        transport::set_cookie(self.cookie(&token, i64::from(self.settings.session_hours) * 3600));
        tracing::info!(event = "login", user_id = %user_id, request_id = %transport::request_id(), "signed in");
        Ok(SessionUser {
            id: user_id.to_string(),
            email,
            full_name: row.get("full_name"),
            role: row.get("role"),
            mfa_enabled,
        })
    }

    pub async fn logout(&self, user: &CurrentUser) -> Result<OkBody> {
        let mut tx = self.db.tx().await?;
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(&user.session_hash)
            .execute(&mut *tx)
            .await
            .map_err(storage("end session"))?;
        self.db.audit(&mut tx, Some(user.id), "logout", None, None).await?;
        tx.commit().await.map_err(storage("end session"))?;
        transport::set_cookie(self.cookie("", 0));
        Ok(OkBody { ok: true })
    }

    pub async fn mfa_enrol(&self, user: &CurrentUser) -> Result<MfaEnrolment> {
        if user.mfa_enabled {
            return Err(AppError::Conflict("Two-factor sign-in is already on".into()));
        }
        let mut bytes = [0u8; 20];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let secret = totp::base32_encode(&bytes);
        let sealed = self.db.seal(&self.db.members.mfa_secret, user.id, &secret)?;
        let mut tx = self.db.tx().await?;
        sqlx::query(
            "INSERT INTO user_mfa (user_id, secret_ciphertext) VALUES ($1, $2)
             ON CONFLICT (user_id) DO UPDATE SET secret_ciphertext = EXCLUDED.secret_ciphertext
             WHERE user_mfa.confirmed_at IS NULL",
        )
        .bind(user.id)
        .bind(sealed.ciphertext())
        .execute(&mut *tx)
        .await
        .map_err(storage("store two-factor secret"))?;
        self.db.audit(&mut tx, Some(user.id), "mfa_enrolment_started", Some(user.id), None).await?;
        tx.commit().await.map_err(storage("store two-factor secret"))?;
        let account = user.email.replace(['?', '&', ' '], "");
        Ok(MfaEnrolment {
            otpauth_url: format!(
                "otpauth://totp/LoanDesk:{account}?secret={secret}&issuer=LoanDesk"
            ),
            secret,
        })
    }

    pub async fn mfa_confirm(&self, user: &CurrentUser, code: &str) -> Result<OkBody> {
        let mut tx = self.db.tx().await?;
        let ciphertext: Option<String> = sqlx::query_scalar(
            "SELECT secret_ciphertext FROM user_mfa WHERE user_id = $1 AND confirmed_at IS NULL FOR UPDATE",
        )
        .bind(user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage("load two-factor secret"))?;
        let ciphertext = ciphertext.ok_or_else(|| bad("Start two-factor set-up first"))?;
        let disclosures = Disclosures::new();
        let secret = self
            .db
            .reveal(
                &self.db.members.mfa_secret,
                user.id,
                ciphertext,
                user.id,
                "confirm-enrolment",
                &disclosures,
            )
            .await?;
        if !totp::verify(&secret, code, unix_now()) {
            drop(disclosures);
            return Err(bad("That code is not valid. Check the authenticator app and try again."));
        }
        sqlx::query("UPDATE user_mfa SET confirmed_at = now() WHERE user_id = $1")
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(storage("confirm two-factor"))?;
        // Other sessions were opened without the second factor: end them.
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND token_hash <> $2")
            .bind(user.id)
            .bind(&user.session_hash)
            .execute(&mut *tx)
            .await
            .map_err(storage("end other sessions"))?;
        self.db.audit(&mut tx, Some(user.id), "mfa_enabled", Some(user.id), None).await?;
        disclosures.persist(&mut tx, user.id).await?;
        tx.commit().await.map_err(storage("confirm two-factor"))?;
        Ok(OkBody { ok: true })
    }
}
