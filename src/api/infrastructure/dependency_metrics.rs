//! Duration and failure metrics for the calls LoanDesk makes to its two
//! dependencies, PostgreSQL and the object store, exported over OTLP beside
//! the framework's HTTP metrics.
//!
//! - `loandesk.dependency.duration_ms` (histogram; `dependency`, `operation`,
//!   `outcome`): every object-store call, and for PostgreSQL the time to get a
//!   connection and open a tenant transaction (which is where a saturated pool
//!   or a slow database shows first), the readiness query and each commit.
//! - `loandesk.dependency.errors` (counter; `operation`): every failed
//!   dependency call, counted where it is logged (`error::storage`).
//!
//! Individual SQL statements are not timed one by one; each is bounded by the
//! statement timeout, and its time is part of the request duration the
//! framework records per route.
use forklaunch_core::telemetry::{MetricType, OpenTelemetryCollector};
use opentelemetry::KeyValue;
use std::{sync::OnceLock, time::Instant};

pub const DURATION: &str = "loandesk.dependency.duration_ms";
pub const ERRORS: &str = "loandesk.dependency.errors";
pub const POSTGRESQL: &str = "postgresql";
pub const OBJECT_STORE: &str = "object_store";

static COLLECTOR: OnceLock<OpenTelemetryCollector> = OnceLock::new();

/// Registers the two instruments. Called once at startup; before that (and in
/// unit tests) recording is a no-op.
pub fn install(mut telemetry: OpenTelemetryCollector) -> Result<(), &'static str> {
    telemetry
        .define_metric(DURATION, MetricType::Histogram)
        .and_then(|()| telemetry.define_metric(ERRORS, MetricType::Counter))
        .map_err(|_| "dependency metrics could not be registered")?;
    let _ = COLLECTOR.set(telemetry);
    Ok(())
}

/// Labels of one measured call.
pub fn labels(dependency: &'static str, operation: &'static str, ok: bool) -> [KeyValue; 3] {
    [
        KeyValue::new("dependency", dependency),
        KeyValue::new("operation", operation),
        KeyValue::new("outcome", if ok { "ok" } else { "error" }),
    ]
}

/// Records how long one dependency call took and whether it succeeded.
pub fn observe(dependency: &'static str, operation: &'static str, started: Instant, ok: bool) {
    if let Some(telemetry) = COLLECTOR.get() {
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        let _ = telemetry.record_metric(DURATION, elapsed, &labels(dependency, operation, ok));
    }
}

/// Counts one failed dependency call.
pub fn failed(operation: &'static str) {
    if let Some(telemetry) = COLLECTOR.get() {
        let _ = telemetry.record_metric(ERRORS, 1.0, &[KeyValue::new("operation", operation)]);
    }
}

/// Runs one dependency call and records its duration and outcome.
pub async fn measured<T, E>(
    dependency: &'static str,
    operation: &'static str,
    call: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let started = Instant::now();
    let result = call.await;
    observe(dependency, operation, started, result.is_ok());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dependency_calls_are_measured_with_their_outcome() {
        // The call's own result is passed through untouched, success or failure.
        assert_eq!(measured(OBJECT_STORE, "head", async { Ok::<_, ()>(7) }).await, Ok(7));
        assert_eq!(
            measured(POSTGRESQL, "begin", async { Err::<(), _>("down") }).await,
            Err("down")
        );
        let ok = labels(OBJECT_STORE, "head", true);
        assert_eq!(ok[0], KeyValue::new("dependency", "object_store"));
        assert_eq!(ok[1], KeyValue::new("operation", "head"));
        assert_eq!(ok[2], KeyValue::new("outcome", "ok"));
        assert_eq!(labels(POSTGRESQL, "begin", false)[2], KeyValue::new("outcome", "error"));
    }
}
