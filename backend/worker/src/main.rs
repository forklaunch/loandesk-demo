//! Document worker. Uses the `documents` table as its queue: claims one
//! pending document at a time (FOR UPDATE SKIP LOCKED, so several workers can
//! run side by side), extracts text from the PDF, pulls out the key figures
//! and saves them on the document and its application.

use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, bail, Context};
use shared::figures::{parse_figures, Figures};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use uuid::Uuid;

const MAX_ATTEMPTS: i32 = 3;
const POLL_INTERVAL: Duration = Duration::from_secs(2);

fn env(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("missing environment variable {name}"))
}

async fn connect_db() -> anyhow::Result<PgPool> {
    let opts = PgConnectOptions::new()
        .host(&env("DB_HOST")?)
        .port(std::env::var("DB_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5432))
        .username(&env("DB_USER")?)
        .password(&env("DB_PASSWORD")?)
        .database(&env("DB_NAME")?);
    let mut last = None;
    for _ in 0..30 {
        match PgPoolOptions::new().max_connections(3).connect_with(opts.clone()).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    bail!("could not connect to the database: {last:?}")
}

struct Job {
    id: Uuid,
    application_id: Uuid,
    storage_key: String,
    attempts: i32,
}

/// Claims the oldest pending document. A document stuck in "processing" for
/// five minutes (worker crashed mid-job) is picked up again.
async fn claim(db: &PgPool) -> Result<Option<Job>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, String, i32)> = sqlx::query_as(
        "UPDATE documents SET status = 'processing', attempts = attempts + 1, locked_at = now()
         WHERE id = (
             SELECT id FROM documents
             WHERE status = 'pending'
                OR (status = 'processing' AND locked_at < now() - interval '5 minutes')
             ORDER BY uploaded_at
             FOR UPDATE SKIP LOCKED
             LIMIT 1)
         RETURNING id, application_id, storage_key, attempts",
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(id, application_id, storage_key, attempts)| Job {
        id,
        application_id,
        storage_key,
        attempts,
    }))
}

async fn extract(docs_dir: &PathBuf, job: &Job) -> anyhow::Result<Figures> {
    let bytes = tokio::fs::read(docs_dir.join(&job.storage_key))
        .await
        .context("reading stored file")?;
    // PDF parsing is CPU-bound and can panic on malformed files; running it
    // on a blocking thread turns a panic into an error for this one document.
    let text = tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&bytes))
        .await
        .map_err(|_| anyhow!("the PDF could not be parsed"))?
        .map_err(|_| anyhow!("the PDF could not be read as text"))?;
    Ok(parse_figures(&text))
}

async fn save(db: &PgPool, job: &Job, f: Figures) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query(
        "UPDATE documents SET status = 'done', processed_at = now(), error = NULL,
                revenue = $2, net_income = $3, total_debt = $4
         WHERE id = $1",
    )
    .bind(job.id)
    .bind(f.revenue)
    .bind(f.net_income)
    .bind(f.total_debt)
    .execute(&mut *tx)
    .await?;
    // A figure found in this document replaces the application's value;
    // a figure this document does not contain leaves it untouched.
    sqlx::query(
        "UPDATE applications SET revenue = COALESCE($2, revenue),
                net_income = COALESCE($3, net_income), total_debt = COALESCE($4, total_debt)
         WHERE id = $1",
    )
    .bind(job.application_id)
    .bind(f.revenue)
    .bind(f.net_income)
    .bind(f.total_debt)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn process(db: &PgPool, docs_dir: &PathBuf, job: Job) -> Result<(), sqlx::Error> {
    if job.attempts > MAX_ATTEMPTS {
        return fail(db, &job, "processing did not finish after several attempts", true).await;
    }
    match extract(docs_dir, &job).await {
        Ok(figures) => {
            save(db, &job, figures).await?;
            // Log ids and which fields were found, never the values.
            tracing::info!(
                document = %job.id,
                revenue = figures.revenue.is_some(),
                net_income = figures.net_income.is_some(),
                total_debt = figures.total_debt.is_some(),
                "document processed"
            );
            Ok(())
        }
        Err(e) => {
            tracing::warn!(document = %job.id, attempt = job.attempts, "extraction failed: {e:#}");
            fail(db, &job, &e.to_string(), job.attempts >= MAX_ATTEMPTS).await
        }
    }
}

async fn fail(db: &PgPool, job: &Job, error: &str, give_up: bool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE documents SET status = $2, error = $3, locked_at = NULL WHERE id = $1")
        .bind(job.id)
        .bind(if give_up { "failed" } else { "pending" })
        .bind(error)
        .execute(db)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let docs_dir = PathBuf::from(env("DOCS_DIR")?);
    let db = connect_db().await?;
    tracing::info!("LoanDesk worker started");

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    loop {
        let wait = match claim(&db).await {
            Ok(Some(job)) => {
                if let Err(e) = process(&db, &docs_dir, job).await {
                    tracing::error!("database error while processing: {e}");
                }
                Duration::ZERO
            }
            Ok(None) => POLL_INTERVAL,
            // Typically the API has not run migrations yet; just retry.
            Err(e) => {
                tracing::warn!("could not poll for documents: {e}");
                Duration::from_secs(5)
            }
        };
        tokio::select! {
            _ = &mut shutdown => break,
            _ = tokio::time::sleep(wait) => {}
        }
    }
    Ok(())
}
