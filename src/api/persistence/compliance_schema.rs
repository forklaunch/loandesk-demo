//! The exporter compiles the same stored models the repository uses, so the
//! published classification (`.forklaunch/compliance.json`) cannot drift from the code.
#[allow(dead_code)]
#[path = "entities/application.rs"]
mod application;
#[allow(dead_code)]
#[path = "entities/audit.rs"]
mod audit;
#[allow(dead_code)]
#[path = "entities/document.rs"]
mod document;
#[allow(dead_code)]
#[path = "entities/owner.rs"]
mod owner;
#[allow(dead_code)]
#[path = "entities/user.rs"]
mod user;
pub use application::StoredApplication;
pub use audit::StoredAuditEntry;
pub use document::StoredDocument;
use forklaunch_core::persistence::compliance::ComplianceEntity;
pub use owner::StoredOwner;
pub use user::{StoredSession, StoredUser, StoredUserMfa};

pub fn declarations() -> Vec<ComplianceEntity> {
    vec![
        ComplianceEntity::of::<StoredApplication>("applications", "postgres-table"),
        ComplianceEntity::of::<StoredOwner>("owners", "postgres-table"),
        ComplianceEntity::of::<StoredDocument>("documents", "postgres-table"),
        ComplianceEntity::of::<StoredUser>("users", "postgres-table"),
        ComplianceEntity::of::<StoredUserMfa>("user_mfa", "postgres-table"),
        ComplianceEntity::of::<StoredSession>("sessions", "postgres-table"),
        ComplianceEntity::of::<StoredAuditEntry>("audit_log", "postgres-table"),
    ]
}
