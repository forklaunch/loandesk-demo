//! Process configuration, resource registration and startup under the framework lifecycle.
use crate::{config::ServiceConfig, domain::services::version, registrations, seed, server};
use forklaunch_axum::native::{self, ApplicationResult, Shutdown};
use std::{env, path::PathBuf};
use tokio::net::TcpListener;

pub async fn run() -> ApplicationResult {
    // `api gen-samples <dir>` writes the sample PDFs and exits; it needs no configuration.
    let arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("gen-samples") {
        let dir = PathBuf::from(arguments.get(1).ok_or("usage: gen-samples <dir>")?);
        for (name, bytes) in shared::pdfgen::sample_documents() {
            std::fs::write(dir.join(name), bytes)?;
            println!("wrote {name}");
        }
        return Ok(());
    }
    // Starts structured JSON logging and, when OTEL_EXPORTER_OTLP_ENDPOINT is
    // set, OTLP export of traces, metrics and logs; flushes them on exit.
    native::run_application(
        "api",
        env!("CARGO_PKG_VERSION"),
        &contracts::registry(),
        start(arguments),
    )
    .await
}

/// A panic anywhere is written to the structured log before the process dies.
fn log_panics() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location =
            info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        tracing::error!(event = "panic", location, backtrace = %std::backtrace::Backtrace::force_capture(), "process panicked");
        default(info);
    }));
}

async fn start(arguments: Vec<String>) -> ApplicationResult {
    log_panics();
    if arguments == ["--migrate"] {
        return Ok(crate::persistence::database::migrate().await?);
    }
    if arguments == ["--rotate-keys"] {
        return Ok(crate::persistence::rotation::run().await?);
    }
    if !arguments.is_empty() {
        return Err("Expected no arguments, --migrate, --rotate-keys or gen-samples <dir>".into());
    }
    let mut config = ServiceConfig::from_env()?;
    config.validate()?;
    let listener = TcpListener::bind(config.bind_addr()).await?;
    config.set_bound_port(listener.local_addr()?.port());
    let state = registrations::register(config).await?;
    if state.settings.seed_demo {
        seed::run(&state).await?;
    }
    tracing::info!(
        event = "startup",
        service = "api",
        version = %version(),
        port = listener.local_addr()?.port(),
        environment = if state.settings.production {
            "production"
        } else if state.settings.hosted_demo {
            "hosted-demo"
        } else {
            "development"
        },
        "LoanDesk API listening"
    );
    let stopping = Shutdown::new();
    let signal = stopping.clone();
    let db = state.db.clone();
    let health = state.health.clone();
    let result = server::serve(state, listener, async move {
        // SIGTERM or Ctrl-C: report not-ready, stop accepting, drain in-flight requests.
        signal.signal().await;
        health.begin_drain();
        tracing::info!(
            event = "shutdown_started",
            service = "api",
            "shutdown signal received; readiness now fails; draining in-flight requests"
        );
    })
    .await;
    db.close().await;
    tracing::info!(
        event = "shutdown_complete",
        service = "api",
        "connections drained and pool closed"
    );
    result
}
