//! Shared framework HTTP/MCP security, LoanDesk's transport layer and the listener lifecycle.
use crate::{api::transport, application, registrations::AppState};
use axum::Router;
use forklaunch_axum::native::ApplicationResult;
use std::future::Future;
use tokio::net::TcpListener;

/// The complete router: ForkLaunch contract routes inside the framework's
/// security and telemetry boundary, inside LoanDesk's transport layer.
pub fn router(state: AppState) -> ApplicationResult<Router> {
    let router = application::http(state.clone())?;
    Ok(router.layer(axum::middleware::from_fn_with_state(state, transport::layer)))
}

/// Serves until `shutdown` resolves, then stops accepting connections and lets
/// in-flight requests finish before returning.
pub async fn serve<F>(state: AppState, listener: TcpListener, shutdown: F) -> ApplicationResult
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, router(state)?).with_graceful_shutdown(shutdown).await?;
    Ok(())
}
