//! Producer side of the extraction worker's queue (ForkLaunch leased PostgreSQL queue).
//!
//! A job is written in the same transaction as the document it belongs to, so
//! a document can never be accepted without its job or the other way round.
//! The payload is encrypted before the queue stores it and carries the id and
//! trace context of the request that queued it.
//!
//! Retries are bounded (`WORKER_MAX_ATTEMPTS`) and delayed. The delay of each
//! job is drawn at random between `WORKER_BACKOFF_SECONDS` and twice that, so
//! jobs that failed together (a storage outage, say) do not all come back at
//! the same instant.
use crate::{
    api::transport,
    error::{AppError, Result, storage},
    persistence::{Db, Tx},
};
use extraction_contracts::jobs::{WorkEvent, WorkInput};
use forklaunch_core::{
    config,
    encryption::{EnvelopeKey, EnvelopeProfile},
    persistence::TenantContext,
};
use forklaunch_worker_database::postgres::PostgresWorkerQueue;
use forklaunch_worker_interface::{
    AsyncWorkerQueue, EncryptedChannel, Job, JobContext, QueueStats, RetryPolicy,
};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

/// Must match the worker's `WORKER_NAME` and channel: they derive the same key.
const WORKER_NAME: &str = "extraction";
const CHANNEL: &str = "work-v1";

/// Retry delays, in seconds, that jobs are spread over: evenly from `base` to
/// twice `base`, never less than one second.
pub fn retry_delays(base_seconds: u64) -> Vec<u64> {
    let base = base_seconds.max(1);
    let mut delays: Vec<u64> = (0..=4).map(|step| base + base * step / 4).collect();
    delays.dedup();
    delays
}

#[derive(Clone)]
pub struct ExtractionQueue {
    /// One handle per retry delay; `enqueue` picks one at random.
    queues: Arc<Vec<PostgresWorkerQueue>>,
    key: Arc<EnvelopeKey>,
    tenant: Uuid,
    name: String,
}

impl ExtractionQueue {
    pub async fn connect(db: &Db) -> Result<Self> {
        // Bounded attempts with a jittered delay between them; exhausted jobs
        // are kept as dead letters for inspection and replay.
        let max_attempts: u32 = config::required("WORKER_MAX_ATTEMPTS")?;
        let backoff_seconds: u64 = config::required("WORKER_BACKOFF_SECONDS")?;
        let tenant = TenantContext::new(db.tenant().to_string())
            .map_err(|_| AppError::InvalidConfig("TENANT_ID is not usable".into()))?;
        let mut queues = Vec::new();
        for delay in retry_delays(backoff_seconds) {
            queues.push(
                PostgresWorkerQueue::new(
                    db.pool().clone(),
                    tenant.clone(),
                    RetryPolicy::new(max_attempts, delay),
                )
                .await
                .map_err(storage("open extraction queue"))?,
            );
        }
        let profile = EnvelopeProfile::new(WORKER_NAME)
            .map_err(|_| AppError::InvalidConfig("invalid worker encryption profile".into()))?;
        let key = EnvelopeKey::from_env(profile).map_err(|_| {
            AppError::InvalidConfig("ENCRYPTION_KEY_HEX is not usable for the queue".into())
        })?;
        Ok(Self {
            queues: Arc::new(queues),
            key: Arc::new(key),
            tenant: db.tenant(),
            name: config::required("WORKER_QUEUE")?,
        })
    }

    /// Queues extraction for one document inside the caller's transaction.
    /// The job id is the document id, so a document is queued at most once.
    pub async fn enqueue(&self, tx: &mut Tx<'_>, document: Uuid, application: Uuid) -> Result<()> {
        let event = WorkEvent {
            context: JobContext {
                tenant_id: self.tenant,
                resource_id: document,
                operation_id: document,
                revision: 1,
            },
            queue: self.name.clone(),
            input: WorkInput {
                resource_id: document,
                revision: 1,
                application_id: application,
                request_id: transport::request_id(),
                traceparent: transport::traceparent().unwrap_or_default(),
            },
        };
        let sealed = EncryptedChannel::<WorkEvent>::new(CHANNEL)
            .and_then(|channel| channel.seal(&self.key, self.tenant, &event))
            .map_err(|_| AppError::Storage("seal extraction job".into()))?;
        let payload = serde_json::to_string(&sealed).map_err(storage("encode extraction job"))?;
        let jittered = rand::thread_rng().gen_range(0..self.queues.len());
        self.queues[jittered]
            .enqueue_in_transaction(tx, Job::new(document.to_string(), &self.name, payload))
            .await
            .map_err(storage("enqueue extraction job"))?;
        Ok(())
    }

    /// Waiting and in-flight job counts, for the readiness report and metrics.
    pub async fn stats(&self) -> Result<QueueStats> {
        self.queues[0].stats(&self.name).await.map_err(storage("read queue depth"))
    }
}

#[cfg(test)]
mod tests {
    use super::retry_delays;

    #[test]
    fn retry_delays_are_spread_between_the_base_and_twice_the_base() {
        assert_eq!(retry_delays(5), [5, 6, 7, 8, 10]);
        assert_eq!(retry_delays(30), [30, 37, 45, 52, 60]);
        // Never an immediate retry, and still more than one choice.
        assert_eq!(retry_delays(0), [1, 2]);
        assert_eq!(retry_delays(1), [1, 2]);
        for base in [1, 2, 5, 30, 600] {
            let delays = retry_delays(base);
            assert!(delays.len() > 1);
            assert!(delays.iter().all(|delay| (base..=base * 2).contains(delay)));
        }
    }
}
