//! Reusable application composition from explicit dependencies; no process startup.
use crate::{api::routes, registrations::AppState};
use forklaunch_axum::native;
use forklaunch_axum::native::Router;
use forklaunch_axum::native::{ApplicationResult, NativeApplication};

pub fn http_router(state: AppState) -> ApplicationResult<Router> {
    let registry = contracts::registry();
    let router = routes::router()?;
    // The OpenAPI document and docs UI describe every route; they are a
    // development aid and are not served in production.
    let router = if state.settings.production {
        router
    } else {
        native::with_docs(router, &registry, &state.config.docs_path, &state.config.version)?
    };
    Ok(router.with_state(state))
}

pub fn http(state: AppState) -> ApplicationResult<NativeApplication> {
    let origin = state.config.origin();
    // Only the configured origin and the exact origins in CORS_ORIGINS may
    // call the API from a browser; `*` is rejected by the framework.
    let cors_origins = state.config.cors_origins.clone();
    let telemetry = state.telemetry.clone();
    NativeApplication::new(
        http_router(state)?,
        contracts::registry(),
        "api",
        env!("CARGO_PKG_VERSION"),
        &origin,
        telemetry,
    )?
    .with_cors_origins(cors_origins)
}
