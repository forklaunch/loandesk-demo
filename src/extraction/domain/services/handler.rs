//! Reads one uploaded PDF, extracts revenue, net income and total debt, and
//! saves them on the document and its application.
//!
//! Delivery is at least once, so everything here is safe to run twice: a
//! document that is already done is acknowledged without touching anything,
//! and the saved figures are the same however often they are written.
//!
//! Only failures that can go away are retried (storage or database trouble).
//! A PDF that cannot be parsed will never parse, so it is marked "could not be
//! read" and acknowledged instead of being retried.
use crate::events;
use forklaunch_core::{
    domain::{BlueprintError, BlueprintResult},
    persistence::{TenantContext, begin_tenant_transaction},
    telemetry::{MetricType, OpenTelemetryCollector},
};
use forklaunch_infrastructure_s3::aws::AwsS3ObjectStore;
use forklaunch_worker_database::runner::JobHandler;
use forklaunch_worker_interface::Job;
use opentelemetry::{
    Context, KeyValue,
    trace::{
        FutureExt, SpanContext, SpanId, SpanKind, Status, TraceContextExt, TraceFlags, TraceId,
        TraceState, Tracer,
    },
};
use shared::{
    figures::{Figures, parse_figures},
    logsafe,
    storage::{MAX_UPLOAD_BYTES, tenant_prefix},
};
use sqlx::PgPool;
use std::time::Instant;
use tracing::Instrument;
use uuid::Uuid;

const JOB_TYPE: &str = "extract_document_figures";
pub const JOB_DURATION: &str = "loandesk.extraction.job.duration_ms";
pub const JOB_OUTCOMES: &str = "loandesk.extraction.jobs";

pub struct ApplicationHandler {
    pool: PgPool,
    tenant: TenantContext,
    store: AwsS3ObjectStore,
    max_attempts: u32,
    telemetry: OpenTelemetryCollector,
}

/// How one delivery ended.
enum Outcome {
    Extracted(Figures),
    /// A previous delivery already finished this document.
    AlreadyDone,
    /// Retrying cannot help; the document is marked failed and the job acknowledged.
    Unreadable(&'static str),
}

/// A failure that may succeed on a later attempt.
struct Transient(&'static str);

/// Logs the failed step with a description of the error that is safe to log
/// (`shared::logsafe`: never `Debug`, never a database message or detail).
fn transient<E: std::fmt::Display + 'static>(step: &'static str) -> impl FnOnce(E) -> Transient {
    move |error| {
        tracing::warn!(
            event = "job_step_failed",
            step,
            error = %logsafe::describe(&error),
            "extraction step failed"
        );
        Transient(step)
    }
}

/// Adds one chunk of a stored file to `bytes`, refusing to hold more than the
/// upload limit in memory whatever the object store returns.
fn append_chunk(bytes: &mut Vec<u8>, chunk: &[u8]) -> bool {
    if (bytes.len() + chunk.len()) as u64 > MAX_UPLOAD_BYTES {
        return false;
    }
    bytes.extend_from_slice(chunk);
    true
}

/// The remote parent named by a W3C `traceparent` ("" or anything malformed
/// gives none, and the job then starts its own trace).
fn remote_parent(traceparent: &str) -> Option<SpanContext> {
    let mut parts = traceparent.split('-');
    let (version, trace, span, flags) =
        (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    if version != "00" || parts.next().is_some() || flags.len() != 2 {
        return None;
    }
    let context = SpanContext::new(
        TraceId::from_hex(trace).ok()?,
        SpanId::from_hex(span).ok()?,
        TraceFlags::new(u8::from_str_radix(flags, 16).ok()?),
        true,
        TraceState::default(),
    );
    (trace.len() == 32 && span.len() == 16 && context.is_valid()).then_some(context)
}

/// Opens the span for one delivery of a job as a child of the request that
/// queued it, so the request and its background work are one trace.
fn job_context<T: Tracer>(tracer: &T, traceparent: &str, attempt: u32) -> Context
where
    T::Span: Send + Sync + 'static,
{
    let parent = remote_parent(traceparent)
        .map_or_else(Context::new, |parent| Context::new().with_remote_span_context(parent));
    let span = tracer
        .span_builder("extraction.job")
        .with_kind(SpanKind::Consumer)
        .with_attributes([
            KeyValue::new("job.type", JOB_TYPE),
            KeyValue::new("job.attempt", i64::from(attempt)),
        ])
        .start_with_context(tracer, &parent);
    parent.with_span(span)
}

impl ApplicationHandler {
    pub fn new(
        pool: PgPool,
        tenant: TenantContext,
        store: AwsS3ObjectStore,
        max_attempts: u32,
        mut telemetry: OpenTelemetryCollector,
    ) -> BlueprintResult<Self> {
        let metric = |_| {
            BlueprintError::new("worker_metrics_unavailable", "Job metrics could not be registered")
        };
        telemetry.define_metric(JOB_DURATION, MetricType::Histogram).map_err(metric)?;
        telemetry.define_metric(JOB_OUTCOMES, MetricType::Counter).map_err(metric)?;
        Ok(Self { pool, tenant, store, max_attempts, telemetry })
    }

    async fn run(
        &self,
        document: Uuid,
        application: Uuid,
        attempt: u32,
    ) -> Result<Outcome, Transient> {
        // Check state before acting: a redelivered job must not redo finished work.
        let mut tx =
            begin_tenant_transaction(&self.pool, &self.tenant).await.map_err(transient("begin"))?;
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT status, storage_key FROM documents WHERE id = $1 AND application_id = $2 FOR UPDATE",
        )
        .bind(document)
        .bind(application)
        .fetch_optional(&mut *tx)
        .await
        .map_err(transient("load document"))?;
        let Some((status, key)) = row else {
            return Ok(Outcome::Unreadable("the document no longer exists"));
        };
        if status == "done" || status == "failed" {
            return Ok(Outcome::AlreadyDone);
        }
        sqlx::query("UPDATE documents SET status = 'processing', attempts = $2, locked_at = now() WHERE id = $1")
            .bind(document)
            .bind(i32::try_from(attempt).unwrap_or(i32::MAX))
            .execute(&mut *tx)
            .await
            .map_err(transient("mark processing"))?;
        tx.commit().await.map_err(transient("mark processing"))?;

        // Files are only ever read from this tenant's prefix.
        if !key.starts_with(&tenant_prefix(self.tenant.id())) {
            return Ok(Outcome::Unreadable("the stored file is not available"));
        }
        // Streamed in bounded chunks with a deadline; the store caps the object size.
        let Some(mut download) =
            self.store.download(&key).await.map_err(transient("open stored file"))?
        else {
            return Ok(Outcome::Unreadable("the stored file is missing"));
        };
        let mut bytes = Vec::new();
        while let Some(chunk) =
            download.next_chunk().await.map_err(transient("read stored file"))?
        {
            if !append_chunk(&mut bytes, &chunk) {
                return Ok(Outcome::Unreadable("the stored file is larger than the 20 MB limit"));
            }
        }

        // PDF parsing is CPU-bound and can panic on malformed files; running it
        // on a blocking thread turns a panic into an error for this one document.
        let text =
            match tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&bytes))
                .await
            {
                Err(_) => return Ok(Outcome::Unreadable("the PDF could not be parsed")),
                Ok(Err(_)) => return Ok(Outcome::Unreadable("the PDF could not be read as text")),
                Ok(Ok(text)) => text,
            };
        let figures = parse_figures(&text);

        let mut tx =
            begin_tenant_transaction(&self.pool, &self.tenant).await.map_err(transient("begin"))?;
        let saved = sqlx::query(
            "UPDATE documents SET status = 'done', processed_at = now(), error = NULL, locked_at = NULL,
                    revenue = $2, net_income = $3, total_debt = $4
             WHERE id = $1 AND status <> 'done'",
        )
        .bind(document)
        .bind(figures.revenue)
        .bind(figures.net_income)
        .bind(figures.total_debt)
        .execute(&mut *tx)
        .await
        .map_err(transient("save figures"))?
        .rows_affected();
        if saved == 1 {
            // A figure found in this document replaces the application's value;
            // a figure this document does not contain leaves it untouched.
            sqlx::query(
                "UPDATE applications SET revenue = COALESCE($2, revenue),
                        net_income = COALESCE($3, net_income), total_debt = COALESCE($4, total_debt)
                 WHERE id = $1",
            )
            .bind(application)
            .bind(figures.revenue)
            .bind(figures.net_income)
            .bind(figures.total_debt)
            .execute(&mut *tx)
            .await
            .map_err(transient("save figures"))?;
        }
        tx.commit().await.map_err(transient("save figures"))?;
        Ok(Outcome::Extracted(figures))
    }

    async fn mark(&self, document: Uuid, status: &str, error: &str) {
        let result = async {
            let mut tx = begin_tenant_transaction(&self.pool, &self.tenant).await?;
            sqlx::query(
                "UPDATE documents SET status = $2, error = $3, locked_at = NULL WHERE id = $1 AND status <> 'done'",
            )
            .bind(document)
            .bind(status)
            .bind(error)
            .execute(&mut *tx)
            .await
            .map_err(|_| forklaunch_core::common::FrameworkError::new("persistence_error", "update failed"))?;
            tx.commit()
                .await
                .map_err(|_| forklaunch_core::common::FrameworkError::new("persistence_error", "commit failed"))
        }
        .await;
        if let Err(error) = result {
            tracing::error!(event = "job_status_not_saved", document = %document, error = %logsafe::describe(&error), "could not record the document status");
        }
    }

    fn measure(&self, outcome: &'static str, started: Instant) {
        let attributes = [KeyValue::new("job.type", JOB_TYPE), KeyValue::new("outcome", outcome)];
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        let _ = self.telemetry.record_metric(JOB_DURATION, elapsed, &attributes);
        let _ = self.telemetry.record_metric(JOB_OUTCOMES, 1.0, &attributes);
    }
}

impl JobHandler for ApplicationHandler {
    async fn process(&self, job: Job) -> BlueprintResult<()> {
        let started = Instant::now();
        // Authenticates tenant, queue and operation before any field is trusted.
        let event = events::open(&job).inspect_err(|_| {
            tracing::error!(event = "job_failed", job_id = %job.id, job_type = JOB_TYPE, attempt = job.attempts, error = "envelope rejected", "job could not be opened");
            self.measure("rejected", started);
        })?;
        let (document, application) = (event.input.resource_id, event.input.application_id);
        let attempt = job.attempts.max(1);
        // The id and trace context of the request that queued this job are on
        // every log line the job writes, and the job's span is a child of that
        // request's span, so the two can be followed as one trace.
        let (queued_by, traceparent) =
            (event.input.request_id.clone(), event.input.traceparent.clone());
        let trace = job_context(
            &opentelemetry::global::tracer("loandesk.extraction"),
            &traceparent,
            attempt,
        );
        let span = tracing::info_span!(
            "extraction_job",
            job_id = %job.id,
            job_type = JOB_TYPE,
            attempt,
            request_id = %queued_by,
            traceparent = %traceparent,
        );
        let traced = trace.clone();
        async {
            match self.run(document, application, attempt).await {
                Ok(Outcome::Extracted(figures)) => {
                    // Log ids and which fields were found, never the values.
                    tracing::info!(
                        event = "job_completed",
                        document = %document,
                        revenue = figures.revenue.is_some(),
                        net_income = figures.net_income.is_some(),
                        total_debt = figures.total_debt.is_some(),
                        duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                        "document processed"
                    );
                    self.measure("completed", started);
                    Ok(())
                }
                Ok(Outcome::AlreadyDone) => {
                    tracing::info!(event = "job_duplicate_ignored", document = %document, "document already processed; nothing to do");
                    self.measure("duplicate", started);
                    Ok(())
                }
                Ok(Outcome::Unreadable(reason)) => {
                    self.mark(document, "failed", reason).await;
                    tracing::warn!(event = "job_failed", document = %document, error = reason, retryable = false, "document could not be read; not retried");
                    self.measure("unreadable", started);
                    Ok(())
                }
                Err(Transient(step)) => {
                    let last = attempt >= self.max_attempts;
                    if last {
                        self.mark(document, "failed", "processing did not finish after several attempts").await;
                    } else {
                        self.mark(document, "pending", "temporary problem reading the document; it will be retried").await;
                    }
                    tracing::error!(
                        event = "job_failed",
                        document = %document,
                        error = step,
                        retryable = true,
                        attempts_exhausted = last,
                        max_attempts = self.max_attempts,
                        "extraction attempt failed"
                    );
                    self.measure(if last { "dead_lettered" } else { "retried" }, started);
                    Err(BlueprintError::new("extraction_transient_failure", step))
                }
            }
        }
        .instrument(span)
        .with_context(trace)
        .await
        .inspect(|()| traced.span().end())
        .inspect_err(|error| {
            traced.span().set_status(Status::error(error.code.clone()));
            traced.span().end();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::noop::NoopTracer;

    const REQUEST: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

    #[test]
    fn a_job_continues_the_trace_of_the_request_that_queued_it() {
        let context = job_context(&NoopTracer::new(), REQUEST, 1);
        let span = context.span();
        let span = span.span_context();
        assert!(span.is_valid());
        assert_eq!(span.trace_id().to_string(), "0af7651916cd43dd8448eb211c80319c");
        assert!(span.is_sampled());
    }

    #[test]
    fn a_job_without_usable_trace_context_starts_its_own_trace() {
        let parent = remote_parent(REQUEST).expect("valid traceparent");
        assert!(parent.is_remote());
        assert_eq!(parent.span_id().to_string(), "b7ad6b7169203331");
        for junk in [
            "",
            "not-a-traceparent",
            "00-00000000000000000000000000000000-b7ad6b7169203331-01",
            "00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01",
            "01-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01-x",
        ] {
            assert!(remote_parent(junk).is_none(), "{junk}");
        }
        assert!(!job_context(&NoopTracer::new(), "", 1).span().span_context().is_valid());
    }

    #[test]
    fn a_stored_file_is_never_held_in_memory_beyond_the_upload_limit() {
        let limit = usize::try_from(MAX_UPLOAD_BYTES).unwrap();
        let mut bytes = vec![0; limit - 2];
        assert!(append_chunk(&mut bytes, &[1, 2]));
        assert_eq!(bytes.len(), limit);
        assert!(!append_chunk(&mut bytes, &[3]));
        assert_eq!(bytes.len(), limit, "the refused chunk was not kept");
    }

    #[test]
    fn a_failed_step_is_logged_without_row_contents() {
        use shared::logsafe::testing::{ROW_VALUES, row_violation};
        let logs = crate::test_logs::capture(|| {
            let Transient(step) = transient("save figures")(row_violation());
            assert_eq!(step, "save figures");
        });
        assert!(logs.contains("job_step_failed") && logs.contains("sqlstate=23514"), "{logs}");
        for value in ROW_VALUES {
            assert!(!logs.contains(value), "`{value}` was written to the log: {logs}");
        }
    }

    #[test]
    fn the_sample_documents_still_yield_their_figures() {
        // Guards the PDF parser upgrade: the generated financial statement
        // must read the same as before.
        let (_, bytes) = shared::pdfgen::sample_documents()
            .into_iter()
            .find(|(name, _)| name.contains("financial-statement"))
            .expect("sample financial statement");
        let text = pdf_extract::extract_text_from_mem(&bytes).expect("readable PDF");
        let figures = parse_figures(&text);
        assert!(figures.revenue.is_some() && figures.net_income.is_some());
        assert!(figures.total_debt.is_some());
    }
}
