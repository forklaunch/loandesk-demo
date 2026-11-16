//! Scheduled data lifecycle and access housekeeping, run by the worker.
//!
//! One pass (`run_once`), every `LIFECYCLE_INTERVAL_SECONDS`:
//!
//! 1. **Retention purge.** Applications past their retention period are
//!    deleted with everything that belongs to them: owners and document rows
//!    (foreign keys, `ON DELETE CASCADE`), the stored PDF of every document
//!    (object storage), and with the application row its extracted figures and
//!    search tokens. Each deleted application leaves one `application_purged`
//!    entry in the audit trail, written in the same transaction as the delete.
//! 2. **Unfinished uploads.** Upload grants that were never confirmed are
//!    removed together with whatever reached object storage.
//! 3. **Expired sessions** are deleted.
//! 4. **Stale accounts.** Accounts unused for `ACCESS_STALE_DAYS` are disabled
//!    and their sessions ended (or only reported, with
//!    `ACCESS_STALE_ACTION=flag`). The last usable admin is never disabled.
//! 5. **Reminders**, as named warnings the alert rules watch: no access review
//!    recorded within `ACCESS_REVIEW_INTERVAL_DAYS`, and an encryption key
//!    older than `ENCRYPTION_KEY_MAX_AGE_DAYS`.
//!
//! The periods are configuration. Their defaults ([`LifecycleConfig::defaults`])
//! are placeholders for a demo and must be confirmed by the people who own
//! record retention and access control (docs/data-classification.md).
//!
//! **Once, not once per instance.** A pass holds a PostgreSQL advisory lock for
//! its whole duration. A second worker, or a pass that is still running when
//! the next one is due, does not get the lock and skips. The lock belongs to
//! the database session, so a crashed worker releases it.
//!
//! **Files first, then rows.** A batch deletes its files, then its rows and
//! audit entries in one transaction. If the transaction fails the rows remain
//! and the next pass repeats the batch (deleting a missing file succeeds). The
//! reverse order could leave a file nobody knows about.
use forklaunch_core::{
    config,
    persistence::{TenantContext, begin_tenant_transaction},
};
use forklaunch_infrastructure_s3::{AsyncObjectStore, aws::AwsS3ObjectStore};
use shared::{logsafe, storage::tenant_prefix};
use sqlx::{PgPool, Row};
use std::{error::Error, time::Duration};
use uuid::Uuid;

/// Advisory lock taken by a pass: (application namespace, job).
const LOCK: (i32, i32) = (4107, 2);
/// Request id recorded on the audit entries this job writes.
const ACTOR: &str = "lifecycle";
/// Batches of one kind per pass. With the default batch size a pass removes at
/// most 1,000 applications of each kind and leaves the rest to the next pass.
const MAX_BATCHES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleAction {
    /// Disable the account and end its sessions.
    Disable,
    /// Leave it enabled; report it in the log and the access review.
    Flag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleConfig {
    /// `LIFECYCLE_INTERVAL_SECONDS`: time between passes. 0 turns the schedule off.
    pub interval: Duration,
    /// `LIFECYCLE_BATCH_SIZE`: applications deleted per transaction.
    pub batch_size: i64,
    /// `RETENTION_DECIDED_APPLICATION_DAYS`: approved or declined applications
    /// are deleted this long after the decision. 0 keeps them.
    pub decided_application_days: i32,
    /// `RETENTION_UNDECIDED_APPLICATION_DAYS`: applications never decided are
    /// deleted this long after they were created. 0 keeps them.
    pub undecided_application_days: i32,
    /// `RETENTION_UNFINISHED_UPLOAD_HOURS`: upload grants never confirmed.
    pub unfinished_upload_hours: i32,
    /// `ACCESS_STALE_DAYS`: an account unused this long is stale.
    pub access_stale_days: i32,
    /// `ACCESS_STALE_ACTION`: `disable` (default) or `flag`.
    pub stale_action: StaleAction,
    /// `ACCESS_REVIEW_INTERVAL_DAYS`: warn when no review was recorded this long.
    pub access_review_interval_days: i32,
    /// `ENCRYPTION_KEY_MAX_AGE_DAYS`: warn when the active key is older.
    pub key_max_age_days: i32,
    /// `ENCRYPTION_KEY_ID`: the active key, as the API registered it.
    pub active_key_id: String,
}

impl LifecycleConfig {
    /// **Demo defaults, not policy.** Chosen so the mechanism runs out of the
    /// box; each must be confirmed or replaced by its owner before LoanDesk
    /// holds real records:
    ///
    /// | Setting | Default | Owner |
    /// | --- | --- | --- |
    /// | decided applications | 2,555 days (7 years) after the decision | records retention |
    /// | undecided applications | 1,095 days (3 years) after creation | records retention |
    /// | unfinished uploads | 24 hours | engineering |
    /// | stale account | 90 days unused, then disabled | access control |
    /// | access review | every 90 days | access control |
    /// | encryption key age | 365 days | security |
    pub fn defaults(active_key_id: String) -> Self {
        Self {
            interval: Duration::from_secs(3600),
            batch_size: 50,
            decided_application_days: 2555,
            undecided_application_days: 1095,
            unfinished_upload_hours: 24,
            access_stale_days: 90,
            stale_action: StaleAction::Disable,
            access_review_interval_days: 90,
            key_max_age_days: 365,
            active_key_id,
        }
    }

    /// Reads the settings; a missing one takes its default, a malformed or
    /// out-of-range one is an error naming it.
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Self::from_lookup(|name| Ok(config::optional::<String>(name)?))
    }

    fn from_lookup(
        lookup: impl Fn(&str) -> Result<Option<String>, Box<dyn Error>>,
    ) -> Result<Self, Box<dyn Error>> {
        let text = |name: &str| -> Result<Option<String>, Box<dyn Error>> {
            Ok(lookup(name)?.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty()))
        };
        let number = |name: &str, default: i32, range: std::ops::RangeInclusive<i32>| {
            let value = match text(name)? {
                None => default,
                Some(value) => {
                    value.parse::<i32>().map_err(|_| format!("{name} must be a whole number"))?
                }
            };
            if range.contains(&value) {
                Ok::<i32, Box<dyn Error>>(value)
            } else {
                Err(format!("{name} must be between {} and {}", range.start(), range.end()).into())
            }
        };
        let defaults =
            Self::defaults(text("ENCRYPTION_KEY_ID")?.ok_or("ENCRYPTION_KEY_ID is required")?);
        Ok(Self {
            interval: Duration::from_secs(u64::try_from(number(
                "LIFECYCLE_INTERVAL_SECONDS",
                3600,
                0..=604_800,
            )?)?),
            batch_size: i64::from(number("LIFECYCLE_BATCH_SIZE", 50, 1..=500)?),
            decided_application_days: number(
                "RETENTION_DECIDED_APPLICATION_DAYS",
                defaults.decided_application_days,
                0..=36_500,
            )?,
            undecided_application_days: number(
                "RETENTION_UNDECIDED_APPLICATION_DAYS",
                defaults.undecided_application_days,
                0..=36_500,
            )?,
            unfinished_upload_hours: number(
                "RETENTION_UNFINISHED_UPLOAD_HOURS",
                defaults.unfinished_upload_hours,
                1..=720,
            )?,
            access_stale_days: number("ACCESS_STALE_DAYS", defaults.access_stale_days, 1..=3650)?,
            stale_action: match text("ACCESS_STALE_ACTION")?.as_deref() {
                None | Some("disable") => StaleAction::Disable,
                Some("flag") => StaleAction::Flag,
                Some(_) => return Err("ACCESS_STALE_ACTION must be disable or flag".into()),
            },
            access_review_interval_days: number(
                "ACCESS_REVIEW_INTERVAL_DAYS",
                defaults.access_review_interval_days,
                1..=3650,
            )?,
            key_max_age_days: number(
                "ENCRYPTION_KEY_MAX_AGE_DAYS",
                defaults.key_max_age_days,
                1..=3650,
            )?,
            active_key_id: defaults.active_key_id,
        })
    }
}

/// What one pass did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    /// Another instance held the lock; nothing was done.
    pub skipped: bool,
    pub applications_purged: u64,
    pub documents_purged: u64,
    pub files_deleted: u64,
    pub unfinished_uploads_removed: u64,
    pub sessions_removed: u64,
    pub stale_accounts: u64,
    pub accounts_disabled: u64,
    pub access_review_overdue: bool,
    pub key_rotation_due: bool,
    /// Steps that failed and will be repeated by the next pass.
    pub failed_steps: u64,
}
impl Report {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "skipped": self.skipped,
            "applications_purged": self.applications_purged,
            "documents_purged": self.documents_purged,
            "files_deleted": self.files_deleted,
            "unfinished_uploads_removed": self.unfinished_uploads_removed,
            "sessions_removed": self.sessions_removed,
            "stale_accounts": self.stale_accounts,
            "accounts_disabled": self.accounts_disabled,
            "access_review_overdue": self.access_review_overdue,
            "key_rotation_due": self.key_rotation_due,
            "failed_steps": self.failed_steps,
        })
        .to_string()
    }
}

/// A step that did not finish. Already logged where it happened.
#[derive(Debug)]
pub struct StepFailed(pub &'static str);

fn failed<E: std::fmt::Display + 'static>(step: &'static str) -> impl FnOnce(E) -> StepFailed {
    move |error| {
        tracing::error!(
            event = "lifecycle_step_failed",
            step,
            error = %logsafe::describe(&error),
            "scheduled maintenance step failed; the next pass repeats it"
        );
        StepFailed(step)
    }
}

/// Which applications a purge selects. The SQL is fixed text; the period is bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expired {
    /// Approved or declined, `days` after the decision.
    Decided,
    /// Never decided, `days` after creation.
    Undecided,
}
impl Expired {
    pub const fn rule(self) -> &'static str {
        match self {
            Self::Decided => "decided",
            Self::Undecided => "undecided",
        }
    }
    /// Selects one batch and locks it; rows another transaction is working on
    /// are skipped rather than waited for. Served by `applications_decided_at_idx`
    /// and `applications_page_idx`.
    pub const fn select(self) -> &'static str {
        match self {
            Self::Decided => {
                "SELECT id FROM applications
                 WHERE status <> 'submitted' AND decided_at < now() - make_interval(days => $1)
                 ORDER BY decided_at LIMIT $2 FOR UPDATE SKIP LOCKED"
            }
            Self::Undecided => {
                "SELECT id FROM applications
                 WHERE status = 'submitted' AND created_at < now() - make_interval(days => $1)
                 ORDER BY created_at LIMIT $2 FOR UPDATE SKIP LOCKED"
            }
        }
    }
}

pub struct Lifecycle {
    pool: PgPool,
    tenant: TenantContext,
    store: AwsS3ObjectStore,
    config: LifecycleConfig,
}

impl Lifecycle {
    pub fn new(
        pool: PgPool,
        tenant: TenantContext,
        store: AwsS3ObjectStore,
        config: LifecycleConfig,
    ) -> Self {
        Self { pool, tenant, store, config }
    }

    /// Deletes stored files. Only keys under this tenant's prefix are touched.
    async fn delete_files(&self, keys: &[String]) -> Result<u64, StepFailed> {
        let prefix = tenant_prefix(self.tenant.id());
        let mut deleted = 0;
        for stored in keys {
            if !stored.starts_with(&prefix) || stored.contains("..") {
                tracing::error!(
                    event = "cross_tenant_storage_key",
                    "a document row points outside the tenant prefix; its file was not deleted"
                );
                return Err(StepFailed("delete stored file"));
            }
            self.store.delete(stored).await.map_err(failed("delete stored file"))?;
            deleted += 1;
        }
        Ok(deleted)
    }

    /// One batch of expired applications: files, then rows and audit entries.
    /// Returns (applications, documents, files) removed.
    async fn purge_batch(&self, kind: Expired, days: i32) -> Result<(u64, u64, u64), StepFailed> {
        let mut tx = begin_tenant_transaction(&self.pool, &self.tenant)
            .await
            .map_err(failed("begin purge"))?;
        let ids: Vec<Uuid> = sqlx::query_scalar(kind.select())
            .bind(days)
            .bind(self.config.batch_size)
            .fetch_all(&mut *tx)
            .await
            .map_err(failed("select expired applications"))?;
        if ids.is_empty() {
            return Ok((0, 0, 0));
        }
        let documents = sqlx::query(
            "SELECT application_id, storage_key FROM documents WHERE application_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(failed("select documents of expired applications"))?;
        let files: Vec<String> = documents.iter().map(|row| row.get("storage_key")).collect();
        let details: Vec<String> = ids
            .iter()
            .map(|id| {
                let count = documents
                    .iter()
                    .filter(|row| row.get::<Uuid, _>("application_id") == *id)
                    .count();
                format!("retention={} days={days} documents={count}", kind.rule())
            })
            .collect();

        let files_deleted = self.delete_files(&files).await?;
        let purged = sqlx::query("DELETE FROM applications WHERE id = ANY($1)")
            .bind(&ids)
            .execute(&mut *tx)
            .await
            .map_err(failed("delete expired applications"))?
            .rows_affected();
        // Who (the scheduled job: no user), what, which record, when, and why.
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id)
             SELECT NULL, 'application_purged', purged.id, purged.detail, $3
             FROM unnest($1::uuid[], $2::text[]) AS purged(id, detail)",
        )
        .bind(&ids)
        .bind(&details)
        .bind(ACTOR)
        .execute(&mut *tx)
        .await
        .map_err(failed("write purge audit entries"))?;
        tx.commit().await.map_err(failed("commit purge"))?;
        Ok((purged, documents.len() as u64, files_deleted))
    }

    async fn purge(&self, kind: Expired, days: i32, totals: &mut Report) -> Result<(), StepFailed> {
        if days == 0 {
            return Ok(());
        }
        for _ in 0..MAX_BATCHES {
            let (applications, documents, files) = self.purge_batch(kind, days).await?;
            if applications == 0 {
                break;
            }
            totals.applications_purged += applications;
            totals.documents_purged += documents;
            totals.files_deleted += files;
            tracing::info!(
                event = "retention_purge",
                retention = kind.rule(),
                days,
                applications,
                documents,
                files,
                "expired applications deleted with their owners, documents and stored files"
            );
        }
        Ok(())
    }

    /// Upload grants that were never confirmed: the row, and the file if one arrived.
    async fn remove_unfinished_uploads(&self, totals: &mut Report) -> Result<(), StepFailed> {
        for _ in 0..MAX_BATCHES {
            let mut tx = begin_tenant_transaction(&self.pool, &self.tenant)
                .await
                .map_err(failed("begin upload clean-up"))?;
            let rows = sqlx::query(
                "SELECT id, storage_key FROM documents
                 WHERE status = 'uploading' AND uploaded_at < now() - make_interval(hours => $1)
                 ORDER BY uploaded_at LIMIT $2 FOR UPDATE SKIP LOCKED",
            )
            .bind(self.config.unfinished_upload_hours)
            .bind(self.config.batch_size)
            .fetch_all(&mut *tx)
            .await
            .map_err(failed("select unfinished uploads"))?;
            if rows.is_empty() {
                break;
            }
            let ids: Vec<Uuid> = rows.iter().map(|row| row.get("id")).collect();
            let files: Vec<String> = rows.iter().map(|row| row.get("storage_key")).collect();
            totals.files_deleted += self.delete_files(&files).await?;
            let removed = sqlx::query("DELETE FROM documents WHERE id = ANY($1)")
                .bind(&ids)
                .execute(&mut *tx)
                .await
                .map_err(failed("delete unfinished uploads"))?
                .rows_affected();
            sqlx::query(
                "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id)
                 SELECT NULL, 'document_upload_expired', id, NULL, $2 FROM unnest($1::uuid[]) AS id",
            )
            .bind(&ids)
            .bind(ACTOR)
            .execute(&mut *tx)
            .await
            .map_err(failed("write upload clean-up audit entries"))?;
            tx.commit().await.map_err(failed("commit upload clean-up"))?;
            totals.unfinished_uploads_removed += removed;
        }
        Ok(())
    }

    async fn remove_expired_sessions(&self, totals: &mut Report) -> Result<(), StepFailed> {
        let mut tx = begin_tenant_transaction(&self.pool, &self.tenant)
            .await
            .map_err(failed("begin session clean-up"))?;
        totals.sessions_removed = sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
            .execute(&mut *tx)
            .await
            .map_err(failed("delete expired sessions"))?
            .rows_affected();
        tx.commit().await.map_err(failed("commit session clean-up"))?;
        Ok(())
    }

    /// Accounts unused for longer than the stale period. With
    /// `StaleAction::Disable` they are disabled and their sessions ended, each
    /// with a `user_disabled` audit entry, except that the most recently used
    /// admin is kept when no other usable admin would remain: a team must not
    /// be locked out of user management by a schedule.
    async fn review_stale_accounts(&self, totals: &mut Report) -> Result<(), StepFailed> {
        let mut tx = begin_tenant_transaction(&self.pool, &self.tenant)
            .await
            .map_err(failed("begin stale account check"))?;
        let stale = sqlx::query(
            "SELECT id, role FROM users
             WHERE disabled_at IS NULL
               AND GREATEST(last_login_at, access_baseline_at) < now() - make_interval(days => $1)
             ORDER BY GREATEST(last_login_at, access_baseline_at) DESC, id
             LIMIT 1000 FOR UPDATE",
        )
        .bind(self.config.access_stale_days)
        .fetch_all(&mut *tx)
        .await
        .map_err(failed("select stale accounts"))?;
        totals.stale_accounts = stale.len() as u64;
        if stale.is_empty() {
            return Ok(());
        }
        if self.config.stale_action == StaleAction::Flag {
            tracing::warn!(
                event = "stale_accounts",
                accounts = totals.stale_accounts,
                stale_after_days = self.config.access_stale_days,
                "accounts have not been used within the stale period; review them"
            );
            return Ok(());
        }
        let usable_admins: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM users
             WHERE role = 'admin' AND disabled_at IS NULL
               AND GREATEST(last_login_at, access_baseline_at) >= now() - make_interval(days => $1)",
        )
        .bind(self.config.access_stale_days)
        .fetch_one(&mut *tx)
        .await
        .map_err(failed("count usable admins"))?;
        let stale_accounts: Vec<(Uuid, bool)> = stale
            .iter()
            .map(|row| (row.get("id"), row.get::<String, _>("role") == "admin"))
            .collect();
        let (disable, kept) = accounts_to_disable(&stale_accounts, usable_admins);
        if let Some(admin) = kept {
            tracing::warn!(
                event = "stale_admin_kept",
                account = %admin,
                stale_after_days = self.config.access_stale_days,
                "the only remaining admin account is stale and was not disabled; review it"
            );
        }
        if disable.is_empty() {
            return Ok(());
        }
        let disabled = sqlx::query(
            "UPDATE users SET disabled_at = now(), disabled_reason = 'inactive' WHERE id = ANY($1)",
        )
        .bind(&disable)
        .execute(&mut *tx)
        .await
        .map_err(failed("disable stale accounts"))?
        .rows_affected();
        sqlx::query("DELETE FROM sessions WHERE user_id = ANY($1)")
            .bind(&disable)
            .execute(&mut *tx)
            .await
            .map_err(failed("end sessions of stale accounts"))?;
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id)
             SELECT NULL, 'user_disabled', id, 'inactive', $2 FROM unnest($1::uuid[]) AS id",
        )
        .bind(&disable)
        .bind(ACTOR)
        .execute(&mut *tx)
        .await
        .map_err(failed("write stale account audit entries"))?;
        tx.commit().await.map_err(failed("commit stale account check"))?;
        totals.accounts_disabled = disabled;
        tracing::warn!(
            event = "stale_accounts_disabled",
            accounts = disabled,
            stale_after_days = self.config.access_stale_days,
            "accounts unused within the stale period were disabled and their sessions ended"
        );
        Ok(())
    }

    /// Warns when the periodic access review or the key rotation is overdue.
    async fn remind(&self, totals: &mut Report) -> Result<(), StepFailed> {
        let mut tx = begin_tenant_transaction(&self.pool, &self.tenant)
            .await
            .map_err(failed("begin reminders"))?;
        // Days since the last recorded review, or since the first account
        // existed when none was ever recorded. NULL: there are no accounts yet.
        let review_age: Option<i32> = sqlx::query_scalar(
            "SELECT (EXTRACT(EPOCH FROM now() - COALESCE(
                        (SELECT max(at) FROM audit_log WHERE action = 'access_review_recorded'),
                        (SELECT min(created_at) FROM users))) / 86400)::int",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(failed("read last access review"))?;
        let key_age: Option<i32> = sqlx::query_scalar(
            "SELECT (EXTRACT(EPOCH FROM now() - min(at)) / 86400)::int FROM audit_log
             WHERE action = 'encryption_key_registered' AND detail = $1",
        )
        .bind(&self.config.active_key_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(failed("read encryption key age"))?;
        tx.commit().await.map_err(failed("commit reminders"))?;

        if let Some(days) =
            review_age.filter(|days| overdue(*days, self.config.access_review_interval_days))
        {
            totals.access_review_overdue = true;
            tracing::warn!(
                event = "access_review_overdue",
                days_since_review = days,
                review_interval_days = self.config.access_review_interval_days,
                "no access review has been recorded within the review interval"
            );
        }
        if let Some(days) = key_age.filter(|days| overdue(*days, self.config.key_max_age_days)) {
            totals.key_rotation_due = true;
            tracing::warn!(
                event = "encryption_key_rotation_due",
                active_days = days,
                max_age_days = self.config.key_max_age_days,
                "the active encryption key is older than its rotation period; rotate it (docs/runbooks/key-rotation.md)"
            );
        }
        Ok(())
    }

    /// One pass, under the advisory lock. A failed step is logged and counted;
    /// the remaining steps still run.
    pub async fn run_once(&self) -> Result<Report, StepFailed> {
        let mut totals = Report::default();
        let mut lock = self.pool.acquire().await.map_err(failed("acquire lock connection"))?;
        let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock($1, $2)")
            .bind(LOCK.0)
            .bind(LOCK.1)
            .fetch_one(&mut *lock)
            .await
            .map_err(failed("take lifecycle lock"))?;
        if !acquired {
            totals.skipped = true;
            tracing::info!(
                event = "lifecycle_skipped",
                "another instance is running the scheduled maintenance; skipped"
            );
            return Ok(totals);
        }
        let config = &self.config;
        let steps = [
            self.purge(Expired::Decided, config.decided_application_days, &mut totals).await,
            self.purge(Expired::Undecided, config.undecided_application_days, &mut totals).await,
            self.remove_unfinished_uploads(&mut totals).await,
            self.remove_expired_sessions(&mut totals).await,
            self.review_stale_accounts(&mut totals).await,
            self.remind(&mut totals).await,
        ];
        totals.failed_steps = steps.iter().filter(|step| step.is_err()).count() as u64;
        let released = sqlx::query("SELECT pg_advisory_unlock($1, $2)")
            .bind(LOCK.0)
            .bind(LOCK.1)
            .execute(&mut *lock)
            .await;
        if released.is_err() {
            // Closing the session releases the lock; never return it to the pool held.
            lock.detach();
        }
        tracing::info!(
            event = "lifecycle_pass",
            applications_purged = totals.applications_purged,
            documents_purged = totals.documents_purged,
            files_deleted = totals.files_deleted,
            unfinished_uploads_removed = totals.unfinished_uploads_removed,
            sessions_removed = totals.sessions_removed,
            stale_accounts = totals.stale_accounts,
            accounts_disabled = totals.accounts_disabled,
            access_review_overdue = totals.access_review_overdue,
            key_rotation_due = totals.key_rotation_due,
            failed_steps = totals.failed_steps,
            "scheduled maintenance pass finished"
        );
        Ok(totals)
    }

    /// Runs a pass now and then every `interval`, until the task is aborted.
    /// A tick that comes due while a pass is still running is dropped, not
    /// queued, so passes never pile up.
    pub async fn run(self) {
        if self.config.interval.is_zero() {
            tracing::warn!(
                event = "lifecycle_disabled",
                "LIFECYCLE_INTERVAL_SECONDS is 0: retention and access housekeeping are not scheduled"
            );
            return;
        }
        let mut ticker = tokio::time::interval(self.config.interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            // Failures are logged where they happen; the next tick tries again.
            let _ = self.run_once().await;
        }
    }
}

/// A period of `limit` days is overdue once more than `limit` whole days have passed.
pub const fn overdue(days: i32, limit: i32) -> bool {
    days > limit
}

/// Which stale accounts to disable. `stale` is ordered most recently used
/// first, each with whether it is an admin. When no usable (enabled, not
/// stale) admin exists, the most recently used stale admin is kept.
pub fn accounts_to_disable(
    stale: &[(Uuid, bool)],
    usable_admins: i64,
) -> (Vec<Uuid>, Option<Uuid>) {
    let kept = if usable_admins > 0 {
        None
    } else {
        stale.iter().find(|(_, admin)| *admin).map(|(id, _)| *id)
    };
    let disable = stale.iter().map(|(id, _)| *id).filter(|id| Some(*id) != kept).collect();
    (disable, kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn config(pairs: &[(&str, &str)]) -> Result<LifecycleConfig, Box<dyn Error>> {
        let mut values: HashMap<String, String> =
            pairs.iter().map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect();
        values.entry("ENCRYPTION_KEY_ID".into()).or_insert_with(|| "key-2026".into());
        LifecycleConfig::from_lookup(|name| Ok(values.get(name).cloned()))
    }

    #[test]
    fn retention_periods_are_defined_per_record_type_and_configurable() {
        // Nothing set: the documented demo defaults.
        let defaults = config(&[]).unwrap();
        assert_eq!(defaults, LifecycleConfig::defaults("key-2026".into()));
        assert_eq!(defaults.decided_application_days, 2555);
        assert_eq!(defaults.undecided_application_days, 1095);
        assert_eq!(defaults.unfinished_upload_hours, 24);
        assert_eq!(defaults.interval, Duration::from_secs(3600));
        // Each period is its own setting.
        let custom = config(&[
            ("RETENTION_DECIDED_APPLICATION_DAYS", "1825"),
            ("RETENTION_UNDECIDED_APPLICATION_DAYS", "0"),
            ("RETENTION_UNFINISHED_UPLOAD_HOURS", "2"),
        ])
        .unwrap();
        assert_eq!(custom.decided_application_days, 1825);
        assert_eq!(custom.undecided_application_days, 0, "0 keeps that record type");
        assert_eq!(custom.unfinished_upload_hours, 2);
        // A typo is an error at startup, not a silently different period.
        for (name, value) in [
            ("RETENTION_DECIDED_APPLICATION_DAYS", "seven years"),
            ("RETENTION_DECIDED_APPLICATION_DAYS", "-1"),
            ("RETENTION_UNFINISHED_UPLOAD_HOURS", "0"),
            ("ACCESS_STALE_DAYS", "0"),
            ("ACCESS_STALE_ACTION", "ignore"),
            ("LIFECYCLE_BATCH_SIZE", "0"),
        ] {
            let error = config(&[(name, value)]).unwrap_err().to_string();
            assert!(error.contains(name), "{error}");
        }
        assert!(
            LifecycleConfig::from_lookup(|_| Ok(None)).is_err(),
            "the active key id is required"
        );
    }

    #[test]
    fn the_purge_selects_only_records_past_their_period_in_bounded_locked_batches() {
        for kind in [Expired::Decided, Expired::Undecided] {
            let sql = kind.select().split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(sql.contains("make_interval(days => $1)"), "the period is a bound value");
            assert!(sql.contains("LIMIT $2"), "one bounded batch");
            assert!(sql.ends_with("FOR UPDATE SKIP LOCKED"), "no waiting on rows in use");
        }
        let decided = Expired::Decided.select();
        assert!(decided.contains("status <> 'submitted'") && decided.contains("decided_at <"));
        let undecided = Expired::Undecided.select();
        assert!(undecided.contains("status = 'submitted'") && undecided.contains("created_at <"));
        assert!(!overdue(90, 90) && overdue(91, 90));
    }

    #[test]
    fn stale_accounts_are_disabled_but_never_the_last_usable_admin() {
        let (officer, old_admin, older_admin) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        // Most recently used first.
        let stale = [(officer, false), (old_admin, true), (older_admin, true)];
        // Another admin is still in use: every stale account is disabled.
        assert_eq!(accounts_to_disable(&stale, 1), (vec![officer, old_admin, older_admin], None));
        // No usable admin: the most recently used stale admin is kept.
        assert_eq!(accounts_to_disable(&stale, 0), (vec![officer, older_admin], Some(old_admin)));
        // No admin among them: nothing to keep.
        assert_eq!(accounts_to_disable(&[(officer, false)], 0), (vec![officer], None));
        assert_eq!(
            config(&[("ACCESS_STALE_ACTION", "flag")]).unwrap().stale_action,
            StaleAction::Flag
        );
    }
}
