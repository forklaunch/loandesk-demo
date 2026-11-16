use forklaunch_core::contract::datetime::{DateTime, Utc};

#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    /// Set while the account is disabled.
    pub disabled_at: Option<DateTime<Utc>>,
}

#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Clone)]
pub struct NewUser {
    #[field(max_length = 254)]
    pub email: String,
    #[field(max_length = 400)]
    pub full_name: String,
    #[field(max_length = 40)]
    pub role: String,
    #[field(max_length = 1024)]
    pub password: String,
}

#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRole {
    #[field(max_length = 40)]
    pub role: String,
}

/// One account in the access review.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessReviewEntry {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    /// `active` or `disabled`.
    pub status: String,
    pub disabled_at: Option<DateTime<Utc>>,
    /// `admin` or `inactive`, when disabled.
    pub disabled_reason: Option<String>,
    /// Whole days since the account was last used (sign-in, creation or re-enabling).
    #[field(min = 0, max = 1000000)]
    pub days_since_last_use: i64,
    /// Active, and unused for longer than `stale_after_days`.
    pub stale: bool,
    #[field(min = 0, max = 1000000)]
    pub open_sessions: i64,
}

/// Who has access, with which role, and when they last used it: what an admin
/// goes through at each periodic access review.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessReview {
    pub generated_at: DateTime<Utc>,
    #[field(min = 0, max = 100000)]
    pub stale_after_days: i64,
    #[field(min = 0, max = 100000)]
    pub review_interval_days: i64,
    pub last_review_at: Option<DateTime<Utc>>,
    /// Id of the admin who recorded the last review.
    pub last_review_by: Option<String>,
    /// No review has been recorded within `review_interval_days`.
    pub review_overdue: bool,
    #[field(min = 0, max = 1000000)]
    pub stale_accounts: i64,
    /// True when there are more accounts than one report holds.
    pub truncated: bool,
    pub users: Vec<AccessReviewEntry>,
}

/// The audit entry written when an admin records that the review was done.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessReviewRecorded {
    pub reviewed_at: DateTime<Utc>,
    #[field(min = 0, max = 1000000)]
    pub accounts: i64,
    #[field(min = 0, max = 1000000)]
    pub stale_accounts: i64,
    #[field(min = 0, max = 1000000)]
    pub disabled_accounts: i64,
}

/// One row of the append-only audit trail: who did what to which record, and
/// when. Rows hold identifiers and fixed action names, never record contents.
#[forklaunch_core::contract::schema]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    #[field(min = 0, max = 9007199254740991)]
    pub id: i64,
    /// When.
    pub at: DateTime<Utc>,
    /// Who: the acting user's id. Absent for the scheduled jobs.
    pub user_id: Option<String>,
    /// What: a fixed action name such as `ssn_revealed` or `user_role_changed`.
    pub action: String,
    /// Which record: the id of the application, owner, document or user.
    pub entity_id: Option<String>,
    /// A fixed qualifier (the new role, the field that was read), never a value.
    pub detail: Option<String>,
    /// The request the entry was written in, to find it in the logs.
    pub request_id: Option<String>,
}
