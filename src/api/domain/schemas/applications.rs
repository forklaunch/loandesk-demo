use super::documents::Document;
use forklaunch_core::contract::datetime::{DateTime, Utc};

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq)]
pub struct Risk {
    pub dscr: Option<f64>,
    pub leverage: Option<f64>,
    pub annual_debt_service: Option<f64>,
    pub flag: Option<String>,
    pub missing: Vec<String>,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq)]
pub struct Application {
    pub id: String,
    pub reference: String,
    pub legal_name: String,
    pub ein: String,
    pub address: String,
    pub industry: String,
    #[field(min = 1, max = 9007199254740991)]
    pub requested_amount: i64,
    /// Free text, stored encrypted. Present on the single-application view only.
    pub purpose: Option<String>,
    pub status: String,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub revenue: Option<i64>,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub net_income: Option<i64>,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub total_debt: Option<i64>,
    pub created_by: String,
    pub created_by_name: String,
    pub created_at: DateTime<Utc>,
    pub decided_by_name: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    /// Free text, stored encrypted. Present on the single-application view only.
    pub decision_note: Option<String>,
    pub risk: Risk,
}

/// Search text travels in the request body, never in the URL: it is often an
/// EIN or a person's name.
#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    #[field(max_length = 100)]
    pub q: String,
    #[field(optional, min = 1, max = 100)]
    pub limit: forklaunch_core::contract::schema::Optional<i64>,
    #[field(optional, max_length = 200)]
    pub cursor: forklaunch_core::contract::schema::Optional<String>,
}

#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Clone)]
pub struct NewOwner {
    #[field(max_length = 400)]
    pub full_name: String,
    #[field(max_length = 20)]
    pub ssn: String,
    #[field(max_length = 10)]
    pub date_of_birth: String,
    pub ownership_percent: f64,
}

#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Clone)]
pub struct NewApplication {
    #[field(max_length = 400)]
    pub legal_name: String,
    #[field(max_length = 20)]
    pub ein: String,
    #[field(max_length = 600)]
    pub address: String,
    #[field(max_length = 200)]
    pub industry: String,
    #[field(min = -9007199254740991, max = 9007199254740991)]
    pub requested_amount: i64,
    #[field(max_length = 4000)]
    pub purpose: String,
    /// Between 1 and 20 owners. The schema macro has no item-count option for
    /// arrays, so the count is enforced in `ApplicationsService::create`
    /// (`MAX_OWNERS`), after the framework's 2 MiB body limit.
    pub owners: Vec<NewOwner>,
}

/// A retried create with the same key returns the first result instead of
/// creating a second application.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyHeaders {
    #[field(rename = "idempotency-key", optional, min_length = 8, max_length = 100)]
    pub idempotency_key: forklaunch_core::contract::schema::Optional<String>,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub id: String,
    pub reference: String,
}

/// An owner as shown on screens: never the full SSN.
#[forklaunch_core::contract::schema]
#[derive(Clone, PartialEq)]
pub struct Owner {
    pub id: String,
    pub full_name: String,
    pub ssn_last4: String,
    pub date_of_birth: String,
    pub ownership_percent: f64,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    pub can_upload: bool,
    pub can_decide: bool,
    pub is_creator: bool,
}

#[forklaunch_core::contract::schema]
#[derive(Clone, PartialEq)]
pub struct ApplicationDetail {
    pub application: Application,
    pub owners: Vec<Owner>,
    pub documents: Vec<Document>,
    pub permissions: Permissions,
}

#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Clone)]
pub struct Decision {
    #[field(max_length = 20)]
    pub decision: String,
    #[field(max_length = 4000)]
    pub note: String,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decided {
    pub status: String,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerPath {
    pub id: String,
    pub owner_id: String,
}

/// The only response that carries a full SSN. Issued by the audited
/// reveal endpoint, one owner at a time.
#[forklaunch_core::contract::schema]
#[derive(Clone)]
pub struct RevealedSsn {
    pub ssn: String,
}

/// Everything held about one loan application, as machine-readable JSON.
#[forklaunch_core::contract::schema]
#[derive(Clone, PartialEq)]
pub struct ApplicationExport {
    pub exported_at: DateTime<Utc>,
    pub application: Application,
    pub owners: Vec<Owner>,
    pub documents: Vec<Document>,
}
