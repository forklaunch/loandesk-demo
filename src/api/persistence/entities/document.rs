//! Metadata of an uploaded PDF. The file itself lives in object storage under
//! `tenants/<tenant>/applications/<application>/<document>.pdf` and carries
//! the same classification as an object tag.
use forklaunch_core::contract::datetime::{DateTime, Utc};
use forklaunch_core::persistence::{compliance::ComplianceMetadata, protected::ProtectedField};
use uuid::Uuid;

#[derive(Debug, ComplianceMetadata)]
#[compliance(entity = "Document", persisted)]
pub struct StoredDocument {
    #[compliance(level = "none")]
    pub id: Uuid,
    #[compliance(level = "none")]
    pub tenant_id: String,
    #[compliance(level = "none")]
    pub application_id: Uuid,
    #[compliance(level = "none")]
    pub doc_type: String,
    /// The name the uploader gave the file: free text, so treated as personal data.
    #[compliance(level = "pii", purpose = "document-filename-v1")]
    pub filename_ciphertext: ProtectedField,
    #[compliance(level = "none")]
    pub size_bytes: i64,
    #[compliance(level = "none")]
    pub storage_key: String,
    /// Classification of the stored file's contents.
    #[compliance(level = "none")]
    pub classification: String,
    #[compliance(level = "none")]
    pub uploaded_by: Uuid,
    #[compliance(level = "none")]
    pub uploaded_at: DateTime<Utc>,
    #[compliance(level = "none")]
    pub status: String,
    #[compliance(level = "none", nullable)]
    pub processed_at: Option<DateTime<Utc>>,
    #[compliance(level = "none", nullable)]
    pub error: Option<String>,
    #[compliance(level = "none", nullable)]
    pub revenue: Option<i64>,
    #[compliance(level = "none", nullable)]
    pub net_income: Option<i64>,
    #[compliance(level = "none", nullable)]
    pub total_debt: Option<i64>,
    #[compliance(level = "none", nullable)]
    pub request_id: Option<String>,
}
