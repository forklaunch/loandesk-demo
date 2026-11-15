use std::fmt;
use std::collections::BTreeMap;
use forklaunch_universal_sdk as runtime;
pub use runtime::{SdkError, SdkAuth, HmacSigner, SdkResponseWithHeaders};
pub type SdkRequest = runtime::SdkRequest;
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
pub struct HealthResponse {
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "recordCount")]
    pub record_count: u64,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "routes")]
    pub routes: Vec<String>,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "service")]
    pub service: String,
    #[serde(deserialize_with = "deserialize_required_field")]
    #[serde(rename = "status")]
    pub status: String,
    #[serde(flatten)]
    pub additional_properties: BTreeMap<String, ::serde_json::Value>,
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
