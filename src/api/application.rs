//! Reusable application composition from explicit dependencies; no process startup.
use crate::{api::routes, registrations::AppState};
use forklaunch_axum::native;
use forklaunch_axum::native::Router;
use forklaunch_axum::native::{ApplicationResult, NativeHttpPolicy, secure, with_telemetry};
use forklaunch_core::mcp::server::{McpServerOptions, mount_http};

/// The framework's default request body limit, kept wherever files do not
/// travel through the API.
const DEFAULT_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

pub fn http_router(state: AppState) -> ApplicationResult<Router> {
    let registry = contracts::registry();
    let router = routes::router()?;
    // The OpenAPI document and docs UI describe every route; they are a
    // development aid and are not served in production.
    let router = if state.settings.serves_api_docs() {
        native::with_docs(router, &registry, &state.config.docs_path, &state.config.version)?
    } else {
        router
    };
    Ok(router.with_state(state))
}

/// Largest request body: the framework default, or one PDF where the hosted
/// demo sends files through the API.
fn max_body_bytes(state: &AppState) -> usize {
    if state.settings.documents_through_api() {
        usize::try_from(state.settings.max_upload_bytes())
            .unwrap_or(DEFAULT_MAX_BODY_BYTES)
            .max(DEFAULT_MAX_BODY_BYTES)
    } else {
        DEFAULT_MAX_BODY_BYTES
    }
}

/// Other origins allowed to call the API from a browser. A hosted demo serves
/// its own front end and has no such caller, so it allows none, whatever
/// `CORS_ORIGINS` holds (the platform appends its own origins to that list).
fn cors_origins(state: &AppState) -> Vec<String> {
    if state.settings.hosted_demo { Vec::new() } else { state.config.cors_origins.clone() }
}

/// The contract routes inside the framework's security and telemetry boundary.
///
/// This is `NativeApplication::new(..).with_cors_origins(..).router()` written
/// out, step for step, because that type has no way to set the request body
/// limit and the hosted demo needs room for one 4 MB PDF.
pub fn http(state: AppState) -> ApplicationResult<Router> {
    let origin = state.config.origin();
    let registry = contracts::registry();
    // Only the configured origin and the exact origins in CORS_ORIGINS may
    // call the API from a browser; `*` is rejected by the framework.
    let policy = NativeHttpPolicy::new(&origin)?
        .with_cors_origins(cors_origins(&state))?
        .with_max_body_bytes(max_body_bytes(&state))?;
    policy.validate_contracts(&registry)?;
    let mut options = McpServerOptions::new("api", env!("CARGO_PKG_VERSION"));
    options.transport = options.transport.with_allowed_origins([origin]);
    let telemetry = state.telemetry.clone();
    let router = mount_http(http_router(state)?, &registry, options)?;
    Ok(with_telemetry(secure(router, policy), telemetry))
}
