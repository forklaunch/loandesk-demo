//! Request and response shapes of the LoanDesk HTTP API.
//!
//! Field names are the wire names the React front end already uses
//! (snake_case). Full SSNs appear in exactly one response type,
//! `applications::RevealedSsn`; everything else carries only the last four.

pub mod applications;
pub mod auth;
pub mod browser_errors;
pub mod documents;
pub mod users;

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
    pub version: String,
    /// Dependency name -> "ok" or "unavailable". Empty for the liveness probe.
    pub checks: Vec<DependencyCheck>,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyCheck {
    pub name: String,
    pub status: String,
}

/// What the front end needs to know about this deployment before sign-in.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSettingsResponse {
    /// A public copy with fictional data: the front end shows the demo banner.
    pub hosted_demo: bool,
    /// Document files are sent to and read from the API, not object storage.
    pub documents_through_api: bool,
    /// Largest PDF accepted, in bytes.
    #[field(min = 1, max = 9007199254740991)]
    pub max_upload_bytes: i64,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdPath {
    pub id: String,
}

/// Keyset pagination. `cursor` is the opaque value from the previous page's
/// `x-next-cursor` response header.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageQuery {
    #[field(optional, min = 1, max = 100)]
    pub limit: forklaunch_core::contract::schema::Optional<i64>,
    #[field(optional, max_length = 200)]
    pub cursor: forklaunch_core::contract::schema::Optional<String>,
}

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeaders {
    /// Present when another page exists.
    #[field(rename = "x-next-cursor")]
    pub next_cursor: Option<String>,
}
