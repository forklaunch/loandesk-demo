use std::fmt;
use std::collections::BTreeMap;
use forklaunch_universal_sdk as runtime;
pub use runtime::{SdkError, SdkAuth, HmacSigner, SdkResponseWithHeaders};
pub type SdkRequest = runtime::SdkRequest;
fn deserialize_optional_field<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: ::serde::Deserializer<'de>,
    T: ::serde::Deserialize<'de>,
{
    T::deserialize(deserializer).map(::std::option::Option::Some)
}
fn deserialize_required_field<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: ::serde::Deserializer<'de>,
    T: ::serde::Deserialize<'de>,
{
    T::deserialize(deserializer)
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct AccessReview {
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "generated_at")]
    pub generated_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "last_review_at")]
    pub last_review_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "last_review_by")]
    pub last_review_by: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "review_interval_days")]
    pub review_interval_days: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "review_overdue")]
    pub review_overdue: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "stale_accounts")]
    pub stale_accounts: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "stale_after_days")]
    pub stale_after_days: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "truncated")]
    pub truncated: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "users")]
    pub users: Vec<AccessReviewEntry>,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct AccessReviewEntry {
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "created_at")]
    pub created_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "days_since_last_use")]
    pub days_since_last_use: i64,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "disabled_at")]
    pub disabled_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "disabled_reason")]
    pub disabled_reason: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "email")]
    pub email: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "last_login_at")]
    pub last_login_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "mfa_enabled")]
    pub mfa_enabled: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "open_sessions")]
    pub open_sessions: i64,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "password_changed_at")]
    pub password_changed_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "role")]
    pub role: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "stale")]
    pub stale: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct AccessReviewRecorded {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "accounts")]
    pub accounts: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "disabled_accounts")]
    pub disabled_accounts: i64,
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "reviewed_at")]
    pub reviewed_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "stale_accounts")]
    pub stale_accounts: i64,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Application {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "address")]
    pub address: String,
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "created_at")]
    pub created_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "created_by")]
    pub created_by: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "created_by_name")]
    pub created_by_name: String,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "decided_at")]
    pub decided_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "decided_by_name")]
    pub decided_by_name: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "decision_note")]
    pub decision_note: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ein")]
    pub ein: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "industry")]
    pub industry: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "legal_name")]
    pub legal_name: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "net_income")]
    pub net_income: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "purpose")]
    pub purpose: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "reference")]
    pub reference: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "requested_amount")]
    pub requested_amount: i64,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "revenue")]
    pub revenue: Option<Option<i64>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "risk")]
    pub risk: Risk,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "total_debt")]
    pub total_debt: Option<Option<i64>>,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct ApplicationDetail {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "application")]
    pub application: Application,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "documents")]
    pub documents: Vec<Document>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "owners")]
    pub owners: Vec<Owner>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "permissions")]
    pub permissions: Permissions,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct ApplicationExport {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "application")]
    pub application: Application,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "documents")]
    pub documents: Vec<Document>,
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "exported_at")]
    pub exported_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "owners")]
    pub owners: Vec<Owner>,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct AuditEntry {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "action")]
    pub action: String,
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "at")]
    pub at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "detail")]
    pub detail: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "entity_id")]
    pub entity_id: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "request_id")]
    pub request_id: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "user_id")]
    pub user_id: Option<Option<String>>,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct BrowserErrorReport {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "class")]
    pub class: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "release")]
    pub release: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "screen")]
    pub screen: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "stack")]
    pub stack: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ChangePassword {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "current_password")]
    pub current_password: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "new_password")]
    pub new_password: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct CompleteDocumentUploadParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: CompleteDocumentUploadParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct CreateApplicationHeaders {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "idempotency-key")]
    pub idempotency_key: Option<CreateApplicationHeadersField0>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Created {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "reference")]
    pub reference: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct DecideApplicationParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: DecideApplicationParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Decided {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "decision")]
    pub decision: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "note")]
    pub note: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct DependencyCheck {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "name")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct DisableUserParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: DisableUserParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Document {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "doc_type")]
    pub doc_type: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "error")]
    pub error: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "filename")]
    pub filename: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "net_income")]
    pub net_income: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "revenue")]
    pub revenue: Option<Option<i64>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "size_bytes")]
    pub size_bytes: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "total_debt")]
    pub total_debt: Option<Option<i64>>,
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "uploaded_at")]
    pub uploaded_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "uploaded_by_name")]
    pub uploaded_by_name: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct DocumentAccepted {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct DownloadDocumentParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: DownloadDocumentParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct DownloadGrant {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "url")]
    pub url: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct EnableUserParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: EnableUserParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct EndpointFailure {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "error")]
    pub error: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ExportApplicationParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: ExportApplicationParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct GetApplicationParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: GetApplicationParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct HealthResponse {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "checks")]
    pub checks: Vec<DependencyCheck>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "service")]
    pub service: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "version")]
    pub version: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListApplicationsQuery {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "limit")]
    pub limit: Option<ListApplicationsQueryField0>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "cursor")]
    pub cursor: Option<ListApplicationsQueryField1>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListApplicationsResponseHeaders {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "x-next-cursor")]
    pub x_next_cursor: Option<Option<String>>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListAuditLogQuery {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "limit")]
    pub limit: Option<ListAuditLogQueryField0>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "cursor")]
    pub cursor: Option<ListAuditLogQueryField1>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListAuditLogResponseHeaders {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "x-next-cursor")]
    pub x_next_cursor: Option<Option<String>>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListUsersQuery {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "limit")]
    pub limit: Option<ListUsersQueryField0>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "cursor")]
    pub cursor: Option<ListUsersQueryField1>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct ListUsersResponseHeaders {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "x-next-cursor")]
    pub x_next_cursor: Option<Option<String>>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "email")]
    pub email: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "password")]
    pub password: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "totp")]
    pub totp: Option<String>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct MfaCode {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "code")]
    pub code: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct MfaEnrolment {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "otpauth_url")]
    pub otpauth_url: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "secret")]
    pub secret: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct NewApplication {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "address")]
    pub address: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ein")]
    pub ein: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "industry")]
    pub industry: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "legal_name")]
    pub legal_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "owners")]
    pub owners: Vec<NewOwner>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "purpose")]
    pub purpose: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "requested_amount")]
    pub requested_amount: i64,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct NewDocument {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "doc_type")]
    pub doc_type: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "filename")]
    pub filename: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "size_bytes")]
    pub size_bytes: i64,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct NewOwner {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "date_of_birth")]
    pub date_of_birth: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ownership_percent")]
    pub ownership_percent: f64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ssn")]
    pub ssn: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct NewUser {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "email")]
    pub email: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "password")]
    pub password: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "role")]
    pub role: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Ok {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ok")]
    pub ok: bool,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Owner {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "date_of_birth")]
    pub date_of_birth: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ownership_percent")]
    pub ownership_percent: f64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ssn_last4")]
    pub ssn_last_4: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Permissions {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "can_decide")]
    pub can_decide: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "can_upload")]
    pub can_upload: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "is_creator")]
    pub is_creator: bool,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct RequestDocumentUploadParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: RequestDocumentUploadParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct RevealOwnerSsnParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: RevealOwnerSsnParamsField0,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "owner_id")]
    pub owner_id: RevealOwnerSsnParamsField1,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct RevealedSsn {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "ssn")]
    pub ssn: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct Risk {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "annual_debt_service")]
    pub annual_debt_service: Option<Option<f64>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "dscr")]
    pub dscr: Option<Option<f64>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "flag")]
    pub flag: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "leverage")]
    pub leverage: Option<Option<f64>>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "missing")]
    pub missing: Vec<String>,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct SearchApplicationsResponseHeaders {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "x-next-cursor")]
    pub x_next_cursor: Option<Option<String>>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "cursor")]
    pub cursor: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_field",
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    #[serde(rename = "limit")]
    pub limit: Option<i64>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "q")]
    pub q: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct SessionUser {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "email")]
    pub email: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "mfa_enabled")]
    pub mfa_enabled: bool,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "role")]
    pub role: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct SetRole {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "role")]
    pub role: String,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
#[serde(deny_unknown_fields)]
pub struct SetUserRoleParams {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: SetUserRoleParamsField0,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct UploadGrant {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "fields")]
    pub fields: BTreeMap<String, String>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "max_bytes")]
    pub max_bytes: i64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "url")]
    pub url: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
#[derive(
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::default::Default,
    ::std::cmp::PartialEq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct User {
    #[serde(with = "::forklaunch_universal_sdk::datetime")]
    #[serde(rename = "created_at")]
    pub created_at: ::forklaunch_universal_sdk::datetime::DateTime<
        ::forklaunch_universal_sdk::datetime::Utc,
    >,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "disabled_at")]
    pub disabled_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "email")]
    pub email: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "id")]
    pub id: String,
    #[serde(with = "::forklaunch_universal_sdk::datetime::optional")]
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    #[serde(rename = "last_login_at")]
    pub last_login_at: Option<
        Option<
            ::forklaunch_universal_sdk::datetime::DateTime<
                ::forklaunch_universal_sdk::datetime::Utc,
            >,
        >,
    >,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "role")]
    pub role: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
}
pub type AccessReviewResponseUsersItem = AccessReviewUsersItem;
pub type AccessReviewUsersItem = AccessReviewEntry;
pub type ApplicationDetailApplication = Application;
pub type ApplicationDetailApplicationRisk = Risk;
pub type ApplicationDetailDocumentsItem = Document;
pub type ApplicationDetailOwnersItem = Owner;
pub type ApplicationDetailPermissions = Permissions;
pub type ApplicationExportApplication = Application;
pub type ApplicationExportApplicationRisk = Risk;
pub type ApplicationExportDocumentsItem = Document;
pub type ApplicationExportOwnersItem = Owner;
pub type ChangePasswordRequestBody = ChangePassword;
pub type CompleteDocumentUploadParamsField0 = String;
pub type ConfirmTwoFactorEnrolmentRequestBody = MfaCode;
pub type CreateApplicationHeadersField0 = String;
pub type CreateApplicationRequestBody = NewApplication;
pub type CreateApplicationRequestBodyOwnersItem = NewApplicationOwnersItem;
pub type CreateUserRequestBody = NewUser;
pub type DecideApplicationParamsField0 = String;
pub type DecideApplicationRequestBody = Decision;
pub type DisableUserParamsField0 = String;
pub type DownloadDocumentParamsField0 = String;
pub type EnableUserParamsField0 = String;
pub type ExportApplicationParamsField0 = String;
pub type ExportApplicationResponseApplication = ApplicationExportApplication;
pub type ExportApplicationResponseApplicationRisk = ApplicationExportApplicationRisk;
pub type ExportApplicationResponseDocumentsItem = ApplicationExportDocumentsItem;
pub type ExportApplicationResponseOwnersItem = ApplicationExportOwnersItem;
pub type GetApplicationParamsField0 = String;
pub type GetApplicationResponseApplication = ApplicationDetailApplication;
pub type GetApplicationResponseApplicationRisk = ApplicationDetailApplicationRisk;
pub type GetApplicationResponseDocumentsItem = ApplicationDetailDocumentsItem;
pub type GetApplicationResponseOwnersItem = ApplicationDetailOwnersItem;
pub type GetApplicationResponsePermissions = ApplicationDetailPermissions;
pub type GetHealthResponseChecksItem = HealthResponseChecksItem;
pub type GetLivenessResponseChecksItem = HealthResponseChecksItem;
pub type GetReadinessResponseChecksItem = HealthResponseChecksItem;
pub type HealthResponseChecksItem = DependencyCheck;
pub type ListApplicationsQueryField0 = i64;
pub type ListApplicationsQueryField1 = String;
pub type ListApplicationsResponseItem = Application;
pub type ListApplicationsResponseItemRisk = Risk;
pub type ListAuditLogQueryField0 = i64;
pub type ListAuditLogQueryField1 = String;
pub type ListAuditLogResponseItem = AuditEntry;
pub type ListUsersQueryField0 = i64;
pub type ListUsersQueryField1 = String;
pub type ListUsersResponseItem = User;
pub type NewApplicationOwnersItem = NewOwner;
pub type ReportBrowserErrorRequestBody = BrowserErrorReport;
pub type RequestDocumentUploadParamsField0 = String;
pub type RequestDocumentUploadRequestBody = NewDocument;
pub type RevealOwnerSsnParamsField0 = String;
pub type RevealOwnerSsnParamsField1 = String;
pub type SearchApplicationsRequestBody = SearchRequest;
pub type SearchApplicationsResponseItem = Application;
pub type SearchApplicationsResponseItemRisk = Risk;
pub type SetUserRoleParamsField0 = String;
pub type SetUserRoleRequestBody = SetRole;
pub type SignInRequestBody = LoginRequest;
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListApplicationsRequest {
    pub params: BTreeMap<String, String>,
    pub query: ListApplicationsQuery,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for ListApplicationsRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ListApplicationsRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ListApplicationsResponses {
    Status200(SdkResponseWithHeaders<Vec<Application>, ListApplicationsResponseHeaders>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct SearchApplicationsRequest<T = SearchApplicationsRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for SearchApplicationsRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for SearchApplicationsRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SearchApplicationsRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum SearchApplicationsResponses {
    Status200(
        SdkResponseWithHeaders<Vec<Application>, SearchApplicationsResponseHeaders>,
    ),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateApplicationRequest<T = CreateApplicationRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: CreateApplicationHeaders,
    pub body: Option<T>,
}
impl<T> Default for CreateApplicationRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for CreateApplicationRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateApplicationRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum CreateApplicationResponses {
    Status201(Box<Created>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct GetApplicationRequest {
    pub params: GetApplicationParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for GetApplicationRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetApplicationRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum GetApplicationResponses {
    Status200(Box<ApplicationDetail>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct DecideApplicationRequest<T = DecideApplicationRequestBody> {
    pub params: DecideApplicationParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for DecideApplicationRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for DecideApplicationRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DecideApplicationRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum DecideApplicationResponses {
    Status200(Box<Decided>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct RevealOwnerSsnRequest {
    pub params: RevealOwnerSsnParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for RevealOwnerSsnRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RevealOwnerSsnRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum RevealOwnerSsnResponses {
    Status200(Box<RevealedSsn>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ExportApplicationRequest {
    pub params: ExportApplicationParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for ExportApplicationRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExportApplicationRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ExportApplicationResponses {
    Status200(Box<ApplicationExport>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct SignInRequest<T = SignInRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for SignInRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for SignInRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignInRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum SignInResponses {
    Status200(Box<SessionUser>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct SignOutRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for SignOutRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignOutRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum SignOutResponses {
    Status200(Box<Ok>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CurrentUserRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for CurrentUserRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CurrentUserRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum CurrentUserResponses {
    Status200(Box<SessionUser>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct StartTwoFactorEnrolmentRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for StartTwoFactorEnrolmentRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StartTwoFactorEnrolmentRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum StartTwoFactorEnrolmentResponses {
    Status200(Box<MfaEnrolment>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ConfirmTwoFactorEnrolmentRequest<T = ConfirmTwoFactorEnrolmentRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for ConfirmTwoFactorEnrolmentRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for ConfirmTwoFactorEnrolmentRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfirmTwoFactorEnrolmentRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ConfirmTwoFactorEnrolmentResponses {
    Status200(Box<Ok>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChangePasswordRequest<T = ChangePasswordRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for ChangePasswordRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for ChangePasswordRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChangePasswordRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ChangePasswordResponses {
    Status200(Box<Ok>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReportBrowserErrorRequest<T = ReportBrowserErrorRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for ReportBrowserErrorRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for ReportBrowserErrorRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReportBrowserErrorRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ReportBrowserErrorResponses {
    Status200(Box<Ok>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct RequestDocumentUploadRequest<T = RequestDocumentUploadRequestBody> {
    pub params: RequestDocumentUploadParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for RequestDocumentUploadRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for RequestDocumentUploadRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestDocumentUploadRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum RequestDocumentUploadResponses {
    Status201(Box<UploadGrant>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompleteDocumentUploadRequest {
    pub params: CompleteDocumentUploadParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for CompleteDocumentUploadRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompleteDocumentUploadRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum CompleteDocumentUploadResponses {
    Status200(Box<DocumentAccepted>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct DownloadDocumentRequest {
    pub params: DownloadDocumentParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for DownloadDocumentRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DownloadDocumentRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum DownloadDocumentResponses {
    Status200(Box<DownloadGrant>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct GetHealthRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for GetHealthRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetHealthRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum GetHealthResponses {
    Status200(Box<HealthResponse>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct GetReadinessRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for GetReadinessRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetReadinessRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum GetReadinessResponses {
    Status200(Box<HealthResponse>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct GetLivenessRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for GetLivenessRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetLivenessRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum GetLivenessResponses {
    Status200(Box<HealthResponse>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListUsersRequest {
    pub params: BTreeMap<String, String>,
    pub query: ListUsersQuery,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for ListUsersRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ListUsersRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ListUsersResponses {
    Status200(SdkResponseWithHeaders<Vec<User>, ListUsersResponseHeaders>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateUserRequest<T = CreateUserRequestBody> {
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for CreateUserRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for CreateUserRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateUserRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum CreateUserResponses {
    Status201(Box<User>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct SetUserRoleRequest<T = SetUserRoleRequestBody> {
    pub params: SetUserRoleParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<T>,
}
impl<T> Default for SetUserRoleRequest<T> {
    fn default() -> Self {
        Self {
            params: Default::default(),
            query: Default::default(),
            headers: Default::default(),
            body: None,
        }
    }
}
impl<T> fmt::Debug for SetUserRoleRequest<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetUserRoleRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("has_body", &self.body.is_some())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum SetUserRoleResponses {
    Status200(Box<User>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct DisableUserRequest {
    pub params: DisableUserParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for DisableUserRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisableUserRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum DisableUserResponses {
    Status200(Box<User>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct EnableUserRequest {
    pub params: EnableUserParams,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for EnableUserRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EnableUserRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum EnableUserResponses {
    Status200(Box<User>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct AccessReviewRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for AccessReviewRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AccessReviewRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum AccessReviewResponses {
    Status200(Box<AccessReview>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct RecordAccessReviewRequest {
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for RecordAccessReviewRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecordAccessReviewRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum RecordAccessReviewResponses {
    Status201(Box<AccessReviewRecorded>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
#[derive(Default, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListAuditLogRequest {
    pub params: BTreeMap<String, String>,
    pub query: ListAuditLogQuery,
    pub headers: BTreeMap<String, String>,
}
impl fmt::Debug for ListAuditLogRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ListAuditLogRequest")
            .field("params", &"<redacted>")
            .field("query", &"<redacted>")
            .field("headers", &"<redacted>")
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub enum ListAuditLogResponses {
    Status200(SdkResponseWithHeaders<Vec<AuditEntry>, ListAuditLogResponseHeaders>),
    Status400(Box<EndpointFailure>),
    Status401(Box<EndpointFailure>),
    Status403(Box<EndpointFailure>),
    Status404(Box<EndpointFailure>),
    Status408(Box<EndpointFailure>),
    Status409(Box<EndpointFailure>),
    Status413(Box<EndpointFailure>),
    Status415(Box<EndpointFailure>),
    Status429(Box<EndpointFailure>),
    Status500(Box<EndpointFailure>),
    Status502(Box<EndpointFailure>),
    Status503(Box<EndpointFailure>),
}
