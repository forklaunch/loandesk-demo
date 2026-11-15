//! Append-only audit trail. Rows hold identifiers and fixed action names, never values.
use forklaunch_core::contract::datetime::{DateTime, Utc};
use forklaunch_core::persistence::compliance::ComplianceMetadata;
use uuid::Uuid;

#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "AuditLog")]
pub struct StoredAuditEntry {
    #[compliance(level = "none")]
    pub id: i64,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "none")]
    pub at: DateTime<Utc>,
    #[compliance(level = "none", nullable)]
    pub user_id: Option<Uuid>,
    #[compliance(level = "none")]
    pub action: String,
    #[compliance(level = "none", nullable)]
    pub entity_id: Option<Uuid>,
    #[compliance(level = "none", nullable)]
    pub detail: Option<String>,
    #[compliance(level = "none", nullable)]
    pub request_id: Option<String>,
}
