//! Makes the queue visible: waiting and in-flight job counts and the number of
//! dead-lettered jobs are exported as metrics and logged, and a non-zero dead
//! letter count is logged as a warning that the alert rules page on.
use forklaunch_core::telemetry::{MetricType, OpenTelemetryCollector};
use forklaunch_worker_database::postgres::PostgresWorkerQueue;
use forklaunch_worker_interface::{AsyncWorkerAdmin, AsyncWorkerQueue};
use opentelemetry::KeyValue;
use std::time::Duration;

pub const QUEUE_DEPTH: &str = "loandesk.extraction.queue.depth";
pub const DEAD_LETTERS: &str = "loandesk.extraction.queue.dead_letters";
const INTERVAL: Duration = Duration::from_secs(15);

pub async fn run(
    queue: PostgresWorkerQueue,
    queue_name: String,
    mut telemetry: OpenTelemetryCollector,
) {
    if telemetry.define_metric(QUEUE_DEPTH, MetricType::Gauge).is_err()
        || telemetry.define_metric(DEAD_LETTERS, MetricType::Gauge).is_err()
    {
        tracing::error!(
            event = "queue_monitor_unavailable",
            "queue metrics could not be registered"
        );
        return;
    }
    let mut ticker = tokio::time::interval(INTERVAL);
    let mut last_dead = usize::MAX;
    loop {
        ticker.tick().await;
        let stats = queue.stats(&queue_name).await;
        let dead = queue.dead_letters(&queue_name, 1000).await;
        match (stats, dead) {
            (Ok(stats), Ok(dead)) => {
                let queued =
                    [KeyValue::new("queue", queue_name.clone()), KeyValue::new("state", "waiting")];
                let active =
                    [KeyValue::new("queue", queue_name.clone()), KeyValue::new("state", "active")];
                let _ = telemetry.record_metric(QUEUE_DEPTH, stats.queued as f64, &queued);
                let _ = telemetry.record_metric(QUEUE_DEPTH, stats.processing as f64, &active);
                let _ = telemetry.record_metric(
                    DEAD_LETTERS,
                    dead.len() as f64,
                    &[KeyValue::new("queue", queue_name.clone())],
                );
                tracing::debug!(event = "queue_depth", queue = %queue_name, waiting = stats.queued, active = stats.processing, dead_letters = dead.len(), "queue depth");
                if dead.len() != last_dead && !dead.is_empty() {
                    tracing::warn!(
                        event = "dead_letter_jobs",
                        queue = %queue_name,
                        dead_letters = dead.len(),
                        "jobs have exhausted their retries; inspect with `extraction --dead-letters 100`"
                    );
                }
                last_dead = dead.len();
            }
            (stats, dead) => tracing::warn!(
                event = "queue_monitor_failed",
                queue = %queue_name,
                stats_ok = stats.is_ok(),
                dead_letters_ok = dead.is_ok(),
                "could not read queue depth"
            ),
        }
    }
}
