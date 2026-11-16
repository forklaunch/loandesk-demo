use forklaunch_core::contract::datetime::{DateTime, Utc};
use std::collections::BTreeMap;

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub doc_type: String,
    pub filename: String,
    #[field(min = 0, max = 9007199254740991)]
    pub size_bytes: i64,
    pub status: String,
    pub error: Option<String>,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub revenue: Option<i64>,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub net_income: Option<i64>,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub total_debt: Option<i64>,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by_name: String,
}

/// Step 1 of an upload: describe the file. The API answers with a short-lived
/// grant to send the bytes straight to object storage.
#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDocument {
    #[field(max_length = 40)]
    pub doc_type: String,
    #[field(max_length = 1024)]
    pub filename: String,
    #[field(min = 1, max = 9007199254740991)]
    pub size_bytes: i64,
}

#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct UploadGrant {
    pub id: String,
    /// POST a multipart form here: every entry of `fields`, then the file as `file`.
    pub url: String,
    pub fields: BTreeMap<String, String>,
    #[field(min = 1, max = 9007199254740991)]
    pub max_bytes: i64,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentAccepted {
    pub id: String,
    pub status: String,
}

#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct DownloadGrant {
    /// Short-lived link to the stored PDF.
    pub url: String,
}
