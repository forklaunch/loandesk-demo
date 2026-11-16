//! Liveness and readiness are separate: liveness never touches a dependency,
//! readiness fails while the database or object store is unreachable.
use crate::{
    domain::schemas::{ClientSettingsResponse, HealthResponse},
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};

/// Readiness, at the path the ForkLaunch platform and Compose health checks probe.
pub async fn health(State(state): State<AppState>) -> Result<HealthResponse, EndpointError> {
    state.health.ready().await.map_err(|_| EndpointError::Unavailable)
}
pub async fn ready(State(state): State<AppState>) -> Result<HealthResponse, EndpointError> {
    state.health.ready().await.map_err(|_| EndpointError::Unavailable)
}
pub async fn live(State(state): State<AppState>) -> Result<HealthResponse, EndpointError> {
    Ok(state.health.live())
}
/// Three facts about the deployment, none of them secret; read by the sign-in page.
pub async fn client_settings(
    State(state): State<AppState>,
) -> Result<ClientSettingsResponse, EndpointError> {
    Ok(ClientSettingsResponse {
        hosted_demo: state.settings.hosted_demo,
        documents_through_api: state.settings.documents_through_api(),
        max_upload_bytes: i64::try_from(state.settings.max_upload_bytes()).unwrap_or(i64::MAX),
    })
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{ClientSettingsResponse, HealthResponse};
    use forklaunch_core::contract::{
        HttpMethod,
        endpoint::{Endpoint, JsonBody, NoBody},
    };

    pub fn health() -> Endpoint<NoBody, JsonBody<HealthResponse>> {
        Endpoint::new(
            "Get Health",
            "Readiness: fails while the database or object store is unreachable",
            HttpMethod::Get,
            "/health",
            200,
        )
        .expect("valid health endpoint")
    }
    pub fn ready() -> Endpoint<NoBody, JsonBody<HealthResponse>> {
        Endpoint::new(
            "Get Readiness",
            "Readiness: fails while the database or object store is unreachable",
            HttpMethod::Get,
            "/api/readyz",
            200,
        )
        .expect("valid readiness endpoint")
    }
    pub fn live() -> Endpoint<NoBody, JsonBody<HealthResponse>> {
        Endpoint::new(
            "Get Liveness",
            "Liveness: the process is up; checks no dependency",
            HttpMethod::Get,
            "/api/healthz",
            200,
        )
        .expect("valid liveness endpoint")
    }
    pub fn client_settings() -> Endpoint<NoBody, JsonBody<ClientSettingsResponse>> {
        Endpoint::new(
            "Get Client Settings",
            "What the front end needs before sign-in: demo banner, upload route and size limit",
            HttpMethod::Get,
            "/api/client-settings",
            200,
        )
        .expect("valid client settings endpoint")
    }
}
