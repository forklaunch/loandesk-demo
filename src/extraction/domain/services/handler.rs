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
use opentelemetry::KeyValue;
use shared::{
    figures::{Figures, parse_figures},
    storage::tenant_prefix,
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

fn transient<E: std::fmt::Debug>(step: &'static str) -> impl FnOnce(E) -> Transient {
    move |error| {
        tracing::warn!(event = "job_step_failed", step, error_detail = ?error, "extraction step failed");
        Transient(step)
    }
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
            bytes.extend_from_slice(&chunk);
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
            tracing::error!(event = "job_status_not_saved", document = %document, error_detail = ?error, "could not record the document status");
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
        // every log line the job writes, so the two can be followed as one.
        let span = tracing::info_span!(
            "extraction_job",
            job_id = %job.id,
            job_type = JOB_TYPE,
            attempt,
            request_id = %event.input.request_id,
            traceparent = %event.input.traceparent,
        );
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
        .await
    }
}
