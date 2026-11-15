//! Staff accounts, sessions and two-factor secrets.
use forklaunch_core::contract::datetime::{DateTime, Utc};
use forklaunch_core::persistence::{compliance::ComplianceMetadata, protected::ProtectedField};
use uuid::Uuid;

/// Staff directory data. Work email and display name are personal data and are
/// declared as such; they are not field-encrypted because sign-in looks users
/// up by email and every list joins on the name. They rely on tenant
/// isolation, role restriction (admin only) and storage-level encryption.
#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "User")]
pub struct StoredUser {
    #[compliance(level = "none")]
    pub id: Uuid,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "pii")]
    pub email: String,
    #[compliance(level = "pii")]
    pub full_name: String,
    #[compliance(level = "none")]
    pub role: String,
    /// Argon2id hash; never leaves the database layer.
    #[compliance(level = "none")]
    pub password_hash: String,
    #[compliance(level = "none")]
    pub failed_logins: i32,
    #[compliance(level = "none", nullable)]
    pub locked_until: Option<DateTime<Utc>>,
    #[compliance(level = "none")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "UserMfa", persisted)]
pub struct StoredUserMfa {
    #[compliance(level = "none")]
    pub user_id: Uuid,
    #[compliance(level = "none")]
    pub tenant_id: String,
    /// TOTP shared secret. A credential, so it is only ever stored sealed.
    #[compliance(level = "pii", purpose = "user-mfa-secret-v1")]
    pub secret_ciphertext: ProtectedField,
    #[compliance(level = "none", nullable)]
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Only a SHA-256 of the session token is stored.
#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "Session")]
pub struct StoredSession {
    #[compliance(level = "none")]
    pub token_hash: String,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "none")]
    pub user_id: Uuid,
    #[compliance(level = "none")]
    pub expires_at: DateTime<Utc>,
}
