//! The one job this worker handles: read an uploaded PDF and extract its key
//! figures. The payload is encrypted before the queue stores it.

use forklaunch_worker_interface::{JobContext, JobPayload};
use uuid::Uuid;
use zeroize::Zeroize;

#[forklaunch_core::contract::schema(rename_all = "camelCase", unknown_fields = "deny")]
pub struct WorkInput {
    /// The document to process. Also the job id, so a document is queued at most once.
    pub resource_id: Uuid,
    #[field(min = 1, max = 9_007_199_254_740_991)]
    pub revision: i64,
    pub application_id: Uuid,
    /// Id of the HTTP request that queued the job; bound to the worker's log lines.
    #[field(max_length = 64)]
    pub request_id: String,
    /// W3C trace context of that request ("" when it had none), so the job's
    /// span joins the request's trace.
    #[field(max_length = 64)]
    pub traceparent: String,
}
impl Zeroize for WorkInput {
    fn zeroize(&mut self) {
        self.request_id.zeroize();
        self.traceparent.zeroize();
    }
}

#[forklaunch_core::contract::schema(rename_all = "camelCase", unknown_fields = "deny")]
pub struct WorkEvent {
    pub context: JobContext,
    pub queue: String,
    pub input: WorkInput,
}
impl Zeroize for WorkEvent {
    fn zeroize(&mut self) {
        self.input.zeroize();
    }
}
impl JobPayload for WorkEvent {
    fn context(&self) -> JobContext {
        self.context
    }
}

/// The producer always authenticates and encrypts before returning transport data.
pub fn seal_work(
    key: &forklaunch_core::encryption::EnvelopeKey,
    tenant: Uuid,
    queue: &str,
    operation: Uuid,
    input: WorkInput,
) -> Result<forklaunch_worker_interface::SealedEvent, forklaunch_worker_interface::ProcessorError> {
    let event = zeroize::Zeroizing::new(WorkEvent {
        context: JobContext {
            tenant_id: tenant,
            resource_id: input.resource_id,
            operation_id: operation,
            revision: input.revision,
        },
        queue: queue.into(),
        input,
    });
    forklaunch_worker_interface::EncryptedChannel::new("work-v1")?.seal(key, tenant, &*event)
}
