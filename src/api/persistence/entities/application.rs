//! Stored loan application.
//!
//! Classification is part of the stored type: a field without a level does not
//! compile, and a classified field can only hold ciphertext.
//!
//! The business's own details (legal name, EIN, address, industry) and the
//! figures taken from its documents are confidential business data, not
//! personal data: they are restricted by role and tenant rather than
//! field-encrypted, which keeps them searchable. Free text is treated as
//! personal data because people type names and account details into it.
use forklaunch_core::contract::datetime::{DateTime, Utc};
use forklaunch_core::persistence::{compliance::ComplianceMetadata, protected::ProtectedField};
use uuid::Uuid;

#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "Application", persisted)]
pub struct StoredApplication {
    #[compliance(level = "none")]
    pub id: Uuid,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "none")]
    pub reference: String,
    #[compliance(level = "none")]
    pub legal_name: String,
    #[compliance(level = "none")]
    pub ein: String,
    #[compliance(level = "none")]
    pub address: String,
    #[compliance(level = "none")]
    pub industry: String,
    #[compliance(level = "none")]
    pub requested_amount: i64,
    #[compliance(level = "pii", purpose = "application-purpose-v1", nullable)]
    pub purpose_ciphertext: Option<ProtectedField>,
    #[compliance(level = "none")]
    pub status: String,
    #[compliance(level = "none", nullable)]
    pub revenue: Option<i64>,
    #[compliance(level = "none", nullable)]
    pub net_income: Option<i64>,
    #[compliance(level = "none", nullable)]
    pub total_debt: Option<i64>,
    #[compliance(level = "none")]
    pub created_by: Uuid,
    #[compliance(level = "none")]
    pub created_at: DateTime<Utc>,
    #[compliance(level = "none", nullable)]
    pub decided_by: Option<Uuid>,
    #[compliance(level = "none", nullable)]
    pub decided_at: Option<DateTime<Utc>>,
    #[compliance(level = "pii", purpose = "application-decision-note-v1", nullable)]
    pub decision_note_ciphertext: Option<ProtectedField>,
    /// Keyed HMAC tokens of words in the encrypted text; not reversible.
    #[compliance(level = "none")]
    pub search_tokens: Vec<String>,
    #[compliance(level = "none", nullable)]
    pub idempotency_key: Option<String>,
}
