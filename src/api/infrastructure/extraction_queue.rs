//! Producer side of the extraction worker's queue (ForkLaunch leased PostgreSQL queue).
//!
//! A job is written in the same transaction as the document it belongs to, so
//! a document can never be accepted without its job or the other way round.
//! The payload is encrypted before the queue stores it and carries the id and
//! trace context of the request that queued it.
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
use std::sync::Arc;
use uuid::Uuid;

/// Must match the worker's `WORKER_NAME` and channel: they derive the same key.
const WORKER_NAME: &str = "extraction";
const CHANNEL: &str = "work-v1";

#[derive(Clone)]
pub struct ExtractionQueue {
    queue: PostgresWorkerQueue,
    key: Arc<EnvelopeKey>,
    tenant: Uuid,
    name: String,
}

impl ExtractionQueue {
    pub async fn connect(db: &Db) -> Result<Self> {
        // Bounded attempts with a delay between them; exhausted jobs are kept
        // as dead letters for inspection and replay.
        let policy = RetryPolicy::new(
            config::required("WORKER_MAX_ATTEMPTS")?,
            config::required("WORKER_BACKOFF_SECONDS")?,
        );
        let tenant = TenantContext::new(db.tenant().to_string())
            .map_err(|_| AppError::InvalidConfig("TENANT_ID is not usable".into()))?;
        let queue = PostgresWorkerQueue::new(db.pool().clone(), tenant, policy)
            .await
            .map_err(storage("open extraction queue"))?;
        let profile = EnvelopeProfile::new(WORKER_NAME)
            .map_err(|_| AppError::InvalidConfig("invalid worker encryption profile".into()))?;
        let key = EnvelopeKey::from_env(profile).map_err(|_| {
            AppError::InvalidConfig("ENCRYPTION_KEY_HEX is not usable for the queue".into())
        })?;
        Ok(Self {
            queue,
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
        self.queue
            .enqueue_in_transaction(tx, Job::new(document.to_string(), &self.name, payload))
            .await
            .map_err(storage("enqueue extraction job"))?;
        Ok(())
    }

    /// Waiting and in-flight job counts, for the readiness report and metrics.
    pub async fn stats(&self) -> Result<QueueStats> {
        self.queue.stats(&self.name).await.map_err(storage("read queue depth"))
    }
}
