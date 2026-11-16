//! Rotating the field-encryption key: `api --rotate-keys`.
//!
//! A rotation has three steps (docs/runbooks/key-rotation.md):
//!
//! 1. Configure the new key as `ENCRYPTION_KEY_ID`/`ENCRYPTION_KEY_HEX` and
//!    move the old one to `ENCRYPTION_PREVIOUS_KEYS_JSON`, then restart the
//!    API and the worker. From here on every new value and search token is
//!    written under the new key, and old rows stay readable.
//! 2. Run `api --rotate-keys` (this module). It walks every table that holds a
//!    protected value, re-seals what is not yet under the active key, rebuilds
//!    the search tokens of those applications, and finishes only when a second
//!    pass finds nothing left under another key.
//! 3. Remove the old key from `ENCRYPTION_PREVIOUS_KEYS_JSON`.
//!
//! Rows are read in keyset order, a bounded batch at a time, and each batch is
//! one transaction: its row locks are short, an interrupted run continues from
//! the start and skips what is already done, and a batch is either rewritten
//! together with its audit entry or not at all. A value is replaced with a
//! compare-and-swap on the ciphertext that was read, so a concurrent write by
//! the running API is never overwritten; the next pass picks it up.
use crate::{
    error::{AppError, Result, storage},
    persistence::{Vault, token_key_id},
};
use forklaunch_core::{
    common::FrameworkResult,
    config,
    persistence::protected::{
        AsyncComplianceAudit, ComplianceAccess, ProtectedMember,
        migration::{MigrationAudit, MigrationPrepared},
    },
};
use sqlx::{Connection, PgConnection, Row};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};
use uuid::Uuid;

/// Rows read and rewritten per transaction.
const BATCH: i64 = 200;
/// Passes over the tables before giving up on concurrent writers.
const MAX_PASSES: u32 = 5;

/// Audit action written by `api --migrate` the first time a key id is active.
/// The scheduled check in the worker measures a key's age from this entry.
pub const KEY_REGISTERED: &str = "encryption_key_registered";
/// Audit action written when a rotation run has finished.
pub const KEY_ROTATED: &str = "encryption_key_rotated";

/// Counts the reads and rewrites the framework reports while re-sealing.
/// They are written to the audit trail once per batch, in the batch's
/// transaction, instead of one row per field.
#[derive(Default)]
pub struct RotationLog {
    read: AtomicU64,
    rewritten: AtomicU64,
}
impl RotationLog {
    fn take(&self) -> (u64, u64) {
        (self.read.swap(0, Ordering::SeqCst), self.rewritten.swap(0, Ordering::SeqCst))
    }
}
impl AsyncComplianceAudit for RotationLog {
    fn record<'a>(
        &'a self,
        _event: ComplianceAccess<'a>,
    ) -> Pin<Box<dyn Future<Output = FrameworkResult<()>> + Send + 'a>> {
        self.read.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}
impl MigrationAudit for RotationLog {
    fn prepared<'a>(
        &'a self,
        _event: MigrationPrepared<'a>,
    ) -> Pin<Box<dyn Future<Output = FrameworkResult<()>> + Send + 'a>> {
        self.rewritten.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

/// What one run did, and what a final read-only pass still found.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub active_key_id: String,
    pub owners: u64,
    pub applications: u64,
    pub documents: u64,
    pub mfa_secrets: u64,
    pub search_indexes_rebuilt: u64,
    /// Rows still under another key after the last pass. Zero when finished.
    pub remaining: u64,
}
impl Report {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "active_key_id": self.active_key_id,
            "owners": self.owners,
            "applications": self.applications,
            "documents": self.documents,
            "mfa_secrets": self.mfa_secrets,
            "search_indexes_rebuilt": self.search_indexes_rebuilt,
            "remaining": self.remaining,
        })
        .to_string()
    }
}

/// Re-seals `stored` when it is not under the active key. Returns the value to
/// keep and whether it changed.
async fn current(
    vault: &Vault,
    member: &ProtectedMember,
    resource: Uuid,
    stored: String,
    operation: Uuid,
    log: &RotationLog,
) -> Result<(String, bool)> {
    if vault.is_current(&stored) {
        return Ok((stored, false));
    }
    Ok((vault.rekey(member, resource, stored, operation, log).await?, true))
}

/// Whether every search token of an application was derived under the active key.
pub fn tokens_current(vault: &Vault, tokens: &[String]) -> bool {
    tokens.iter().all(|token| token_key_id(token) == Some(vault.active_key_id()))
}

async fn audit_batch(connection: &mut PgConnection, action: &str, detail: &str) -> Result<()> {
    sqlx::query("INSERT INTO audit_log (user_id, action, entity_id, detail, request_id) VALUES (NULL, $1, NULL, $2, 'key-rotation')")
        .bind(action)
        .bind(detail)
        .execute(&mut *connection)
        .await
        .map_err(storage("write rotation audit entry"))?;
    Ok(())
}

/// One pass over applications and their owners. With `write` false nothing is
/// changed and the count is of rows that still need work.
async fn applications_pass(
    connection: &mut PgConnection,
    vault: &Vault,
    operation: Uuid,
    write: bool,
    totals: &mut Report,
) -> Result<u64> {
    let members = vault.members.clone();
    let log = RotationLog::default();
    let mut stale_rows = 0_u64;
    let mut after: Option<Uuid> = None;
    loop {
        let mut tx = connection.begin().await.map_err(storage("begin rotation batch"))?;
        let applications = sqlx::query(
            "SELECT id, purpose_ciphertext, decision_note_ciphertext, search_tokens
             FROM applications WHERE ($1::uuid IS NULL OR id > $1) ORDER BY id LIMIT $2",
        )
        .bind(after)
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read applications for rotation"))?;
        let Some(last) = applications.last() else { break };
        after = Some(last.get("id"));
        let ids: Vec<Uuid> = applications.iter().map(|row| row.get("id")).collect();
        let owner_rows = sqlx::query(
            "SELECT id, application_id, full_name_ciphertext, ssn_ciphertext, date_of_birth_ciphertext
             FROM owners WHERE application_id = ANY($1) ORDER BY application_id, id",
        )
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read owners for rotation"))?;
        let mut owners: BTreeMap<Uuid, Vec<(Uuid, [String; 3])>> = BTreeMap::new();
        for row in &owner_rows {
            owners.entry(row.get("application_id")).or_default().push((
                row.get("id"),
                [
                    row.get("full_name_ciphertext"),
                    row.get("ssn_ciphertext"),
                    row.get("date_of_birth_ciphertext"),
                ],
            ));
        }

        let (mut owners_done, mut applications_done, mut indexes_done) = (0_u64, 0_u64, 0_u64);
        for row in &applications {
            let id: Uuid = row.get("id");
            let purpose: Option<String> = row.get("purpose_ciphertext");
            let note: Option<String> = row.get("decision_note_ciphertext");
            let tokens: Vec<String> = row.get("search_tokens");
            let application_owners = owners.remove(&id).unwrap_or_default();
            let fields_stale = purpose.iter().chain(note.iter()).any(|v| !vault.is_current(v));
            let owners_stale = application_owners
                .iter()
                .filter(|(_, fields)| fields.iter().any(|v| !vault.is_current(v)))
                .count() as u64;
            let index_stale = !tokens_current(vault, &tokens);
            if !fields_stale && owners_stale == 0 && !index_stale {
                continue;
            }
            stale_rows += 1 + owners_stale;
            if !write {
                continue;
            }

            // The search tokens are derived from the purpose and the owners'
            // names, so those are read here whether or not they need re-sealing.
            let mut names = Vec::with_capacity(application_owners.len());
            for (owner, [name, ssn, dob]) in application_owners {
                names.push(
                    vault
                        .reveal(
                            &members.owner_name,
                            owner,
                            name.clone(),
                            Uuid::nil(),
                            "rotate-encryption-key",
                            &log,
                        )
                        .await?,
                );
                let (new_name, a) =
                    current(vault, &members.owner_name, owner, name.clone(), operation, &log)
                        .await?;
                let (new_ssn, b) =
                    current(vault, &members.owner_ssn, owner, ssn.clone(), operation, &log).await?;
                let (new_dob, c) =
                    current(vault, &members.owner_dob, owner, dob.clone(), operation, &log).await?;
                if a || b || c {
                    let changed = sqlx::query(
                        "UPDATE owners SET full_name_ciphertext = $2, ssn_ciphertext = $3,
                                date_of_birth_ciphertext = $4
                         WHERE id = $1 AND full_name_ciphertext = $5 AND ssn_ciphertext = $6
                           AND date_of_birth_ciphertext = $7",
                    )
                    .bind(owner)
                    .bind(new_name)
                    .bind(new_ssn)
                    .bind(new_dob)
                    .bind(name)
                    .bind(ssn)
                    .bind(dob)
                    .execute(&mut *tx)
                    .await
                    .map_err(storage("re-seal owner"))?
                    .rows_affected();
                    owners_done += changed;
                }
            }
            let plain_purpose = match &purpose {
                Some(stored) => Some(
                    vault
                        .reveal(
                            &members.purpose,
                            id,
                            stored.clone(),
                            Uuid::nil(),
                            "rotate-encryption-key",
                            &log,
                        )
                        .await?,
                ),
                None => None,
            };
            let new_tokens = vault.index_tokens(
                plain_purpose.iter().map(String::as_str).chain(names.iter().map(String::as_str)),
            )?;
            drop(plain_purpose);
            drop(names);
            let new_purpose = match purpose.clone() {
                Some(stored) => {
                    Some(current(vault, &members.purpose, id, stored, operation, &log).await?.0)
                }
                None => None,
            };
            let new_note = match note.clone() {
                Some(stored) => Some(
                    current(vault, &members.decision_note, id, stored, operation, &log).await?.0,
                ),
                None => None,
            };
            // Compare-and-swap: a decision recorded by the running API since
            // this batch was read is left alone and handled by the next pass.
            let changed = sqlx::query(
                "UPDATE applications SET purpose_ciphertext = $2, decision_note_ciphertext = $3,
                        search_tokens = $4
                 WHERE id = $1 AND purpose_ciphertext IS NOT DISTINCT FROM $5
                   AND decision_note_ciphertext IS NOT DISTINCT FROM $6",
            )
            .bind(id)
            .bind(new_purpose)
            .bind(new_note)
            .bind(&new_tokens)
            .bind(purpose)
            .bind(note)
            .execute(&mut *tx)
            .await
            .map_err(storage("re-seal application"))?
            .rows_affected();
            applications_done += changed;
            indexes_done += changed;
        }
        if write && owners_done + applications_done > 0 {
            let (read, rewritten) = log.take();
            audit_batch(
                &mut tx,
                "protected_fields_rekeyed",
                &format!(
                    "applications={applications_done} owners={owners_done} fields_read={read} fields_rewritten={rewritten} to_key={}",
                    vault.active_key_id()
                ),
            )
            .await?;
        }
        tx.commit().await.map_err(storage("commit rotation batch"))?;
        totals.owners += owners_done;
        totals.applications += applications_done;
        totals.search_indexes_rebuilt += indexes_done;
    }
    Ok(stale_rows)
}

/// One pass over a table with a single protected column keyed by a UUID.
async fn column_pass(
    connection: &mut PgConnection,
    vault: &Vault,
    operation: Uuid,
    write: bool,
    table: Table,
    done: &mut u64,
) -> Result<u64> {
    let (select, update, member, label) = match table {
        Table::Documents => (
            "SELECT id, filename_ciphertext AS stored FROM documents
             WHERE ($1::uuid IS NULL OR id > $1) ORDER BY id LIMIT $2",
            "UPDATE documents SET filename_ciphertext = $2 WHERE id = $1 AND filename_ciphertext = $3",
            vault.members.filename.clone(),
            "documents",
        ),
        Table::UserMfa => (
            "SELECT user_id AS id, secret_ciphertext AS stored FROM user_mfa
             WHERE ($1::uuid IS NULL OR user_id > $1) ORDER BY user_id LIMIT $2",
            "UPDATE user_mfa SET secret_ciphertext = $2 WHERE user_id = $1 AND secret_ciphertext = $3",
            vault.members.mfa_secret.clone(),
            "mfa_secrets",
        ),
    };
    let log = RotationLog::default();
    let mut stale_rows = 0_u64;
    let mut after: Option<Uuid> = None;
    loop {
        let mut tx = connection.begin().await.map_err(storage("begin rotation batch"))?;
        let rows = sqlx::query(select)
            .bind(after)
            .bind(BATCH)
            .fetch_all(&mut *tx)
            .await
            .map_err(storage("read rows for rotation"))?;
        let Some(last) = rows.last() else { break };
        after = Some(last.get("id"));
        let mut batch_done = 0_u64;
        for row in &rows {
            let id: Uuid = row.get("id");
            let stored: String = row.get("stored");
            if vault.is_current(&stored) {
                continue;
            }
            stale_rows += 1;
            if !write {
                continue;
            }
            let replacement = vault.rekey(&member, id, stored.clone(), operation, &log).await?;
            batch_done += sqlx::query(update)
                .bind(id)
                .bind(replacement)
                .bind(stored)
                .execute(&mut *tx)
                .await
                .map_err(storage("re-seal row"))?
                .rows_affected();
        }
        if write && batch_done > 0 {
            let (read, rewritten) = log.take();
            audit_batch(
                &mut tx,
                "protected_fields_rekeyed",
                &format!(
                    "{label}={batch_done} fields_read={read} fields_rewritten={rewritten} to_key={}",
                    vault.active_key_id()
                ),
            )
            .await?;
        }
        tx.commit().await.map_err(storage("commit rotation batch"))?;
        *done += batch_done;
    }
    Ok(stale_rows)
}

#[derive(Clone, Copy)]
enum Table {
    Documents,
    UserMfa,
}

async fn pass(
    connection: &mut PgConnection,
    vault: &Vault,
    operation: Uuid,
    write: bool,
    totals: &mut Report,
) -> Result<u64> {
    let mut stale = applications_pass(connection, vault, operation, write, totals).await?;
    let mut documents = 0;
    stale +=
        column_pass(connection, vault, operation, write, Table::Documents, &mut documents).await?;
    let mut mfa = 0;
    stale += column_pass(connection, vault, operation, write, Table::UserMfa, &mut mfa).await?;
    totals.documents += documents;
    totals.mfa_secrets += mfa;
    Ok(stale)
}

/// Re-seals everything that is not under the active key and rebuilds the
/// search tokens that depend on it. Runs with the schema-owner role on one
/// connection bound to the tenant (as `--migrate` does).
pub async fn rotate(connection: &mut PgConnection, tenant: Uuid) -> Result<Report> {
    let vault = Vault::from_env(tenant)?;
    let operation = Uuid::new_v4();
    let mut totals =
        Report { active_key_id: vault.active_key_id().to_owned(), ..Report::default() };
    for _ in 0..MAX_PASSES {
        if pass(connection, &vault, operation, true, &mut totals).await? == 0 {
            break;
        }
    }
    // Read-only confirmation: nothing may be left under another key.
    totals.remaining = pass(connection, &vault, operation, false, &mut Report::default()).await?;
    register_active_key(connection, tenant).await?;
    let rewritten = totals.owners + totals.applications + totals.documents + totals.mfa_secrets;
    if totals.remaining == 0 && rewritten > 0 {
        audit_batch(
            connection,
            KEY_ROTATED,
            &format!(
                "active_key={} owners={} applications={} documents={} mfa_secrets={}",
                totals.active_key_id,
                totals.owners,
                totals.applications,
                totals.documents,
                totals.mfa_secrets
            ),
        )
        .await?;
    }
    tracing::info!(
        event = "encryption_key_rotation",
        owners = totals.owners,
        applications = totals.applications,
        documents = totals.documents,
        mfa_secrets = totals.mfa_secrets,
        search_indexes_rebuilt = totals.search_indexes_rebuilt,
        remaining = totals.remaining,
        "protected fields re-sealed under the active key"
    );
    Ok(totals)
}

/// Records the first time a key id is the active one. The scheduled check in
/// the worker (`extraction/lifecycle.rs`) warns when that is longer ago than
/// `ENCRYPTION_KEY_MAX_AGE_DAYS`.
pub async fn register_active_key(connection: &mut PgConnection, _tenant: Uuid) -> Result<()> {
    let active: String = config::required("ENCRYPTION_KEY_ID")?;
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id)
         SELECT NULL, $1, NULL, $2, 'migrate'
         WHERE NOT EXISTS (SELECT 1 FROM audit_log WHERE action = $1 AND detail = $2)",
    )
    .bind(KEY_REGISTERED)
    .bind(&active)
    .execute(&mut *connection)
    .await
    .map_err(storage("record active encryption key"))?;
    Ok(())
}

/// `api --rotate-keys`: connects like `--migrate` and prints the report.
pub async fn run() -> Result<()> {
    let (pool, tenant) = super::database::migration_pool().await?;
    let result = async {
        let mut connection =
            pool.acquire().await.map_err(storage("acquire rotation connection"))?;
        sqlx::query("SELECT set_config('forklaunch.tenant_id', $1, false)")
            .bind(tenant.to_string())
            .execute(&mut *connection)
            .await
            .map_err(storage("bind rotation tenant"))?;
        let totals = rotate(&mut connection, tenant).await?;
        println!("{}", totals.to_json());
        if totals.remaining > 0 {
            return Err(AppError::Storage(
                "rows remain under a previous key: run --rotate-keys again before retiring it"
                    .into(),
            ));
        }
        Ok(())
    }
    .await;
    pool.close().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::stored_key_id;

    const NAMESPACE: &str = "loandesk-records";
    const OLD: [u8; 32] = [0x11; 32];
    const NEW: [u8; 32] = [0x22; 32];

    fn tenant() -> Uuid {
        Uuid::parse_str("6f1d5a0e-58f4-4c58-9a51-0c2f6b1d7c11").unwrap()
    }
    /// Before the rotation: one key.
    fn before() -> Vault {
        Vault::with_keys(tenant(), NAMESPACE, ("key-2025", OLD), Vec::new()).unwrap()
    }
    /// During the rotation: the new key is active, the old one still readable.
    fn during() -> Vault {
        Vault::with_keys(tenant(), NAMESPACE, ("key-2026", NEW), vec![("key-2025".into(), OLD)])
            .unwrap()
    }
    /// After the old key has been retired.
    fn after() -> Vault {
        Vault::with_keys(tenant(), NAMESPACE, ("key-2026", NEW), Vec::new()).unwrap()
    }
    async fn open(
        vault: &Vault,
        member: &ProtectedMember,
        id: Uuid,
        stored: &str,
    ) -> Result<String> {
        let log = RotationLog::default();
        vault.reveal(member, id, stored.to_owned(), Uuid::nil(), "test", &log).await
    }

    #[tokio::test]
    async fn encryption_keys_are_rotated_rows_are_re_encrypted_and_the_old_key_can_be_retired() {
        let owner = Uuid::new_v4();
        let old = before();
        let members = old.members.clone();
        let name = old.seal(&members.owner_name, owner, "Eleanor Whitfield").unwrap();
        let ssn = old.seal(&members.owner_ssn, owner, "900114321").unwrap();
        assert_eq!(stored_key_id(name.ciphertext()).as_deref(), Some("key-2025"));
        assert_eq!(stored_key_id(ssn.ciphertext()).as_deref(), Some("key-2025"), "pd1 wrapper");

        // Step 1: new key active, old key kept for reads. Old rows still open.
        let rotating = during();
        assert!(!rotating.is_current(name.ciphertext()));
        assert_eq!(
            open(&rotating, &members.owner_name, owner, name.ciphertext()).await.unwrap(),
            "Eleanor Whitfield"
        );

        // Step 2: re-seal. The value and the SSN's last four are unchanged.
        let log = RotationLog::default();
        let operation = Uuid::new_v4();
        let (new_name, changed) = current(
            &rotating,
            &members.owner_name,
            owner,
            name.ciphertext().to_owned(),
            operation,
            &log,
        )
        .await
        .unwrap();
        let (new_ssn, _) = current(
            &rotating,
            &members.owner_ssn,
            owner,
            ssn.ciphertext().to_owned(),
            operation,
            &log,
        )
        .await
        .unwrap();
        assert!(changed && rotating.is_current(&new_name) && rotating.is_current(&new_ssn));
        assert_ne!(new_name, name.ciphertext());
        assert_eq!(log.take(), (2, 2), "each re-seal is one audited read and one audited rewrite");
        // Running it again changes nothing: the rotation is idempotent.
        let (again, changed) =
            current(&rotating, &members.owner_name, owner, new_name.clone(), operation, &log)
                .await
                .unwrap();
        assert!(!changed && again == new_name);

        // Step 3: the old key is gone. Re-sealed rows open; untouched rows do not.
        let retired = after();
        assert_eq!(
            open(&retired, &members.owner_name, owner, &new_name).await.unwrap(),
            "Eleanor Whitfield"
        );
        assert_eq!(open(&retired, &members.owner_ssn, owner, &new_ssn).await.unwrap(), "900114321");
        assert_eq!(retired.ssn_last4(owner, new_ssn).unwrap(), "4321");
        assert!(open(&retired, &members.owner_name, owner, name.ciphertext()).await.is_err());
        // A value sealed for one row cannot be moved to another.
        assert!(open(&retired, &members.owner_name, Uuid::new_v4(), &new_name).await.is_err());
    }

    #[test]
    fn search_tokens_are_rebuilt_under_the_new_key_and_old_ones_match_until_then() {
        let old_tokens = before().index_tokens(["Eleanor Whitfield"]).unwrap();
        let rotating = during();
        assert!(!tokens_current(&rotating, &old_tokens), "tokens under the old key are stale");
        // While rotating, a query carries both generations, so a row that has
        // not been re-indexed yet is still found.
        let query = rotating.query_tokens("whitfield").unwrap().unwrap();
        let previous = query.previous.clone().expect("the previous key is still configured");
        assert!(previous.iter().all(|token| old_tokens.contains(token)));
        assert!(!query.active.iter().any(|token| old_tokens.contains(token)));

        let new_tokens = rotating.index_tokens(["Eleanor Whitfield"]).unwrap();
        assert!(tokens_current(&rotating, &new_tokens));
        assert!(query.active.iter().all(|token| new_tokens.contains(token)));

        // After the old key is retired a query carries one generation, and the
        // rebuilt index still matches it.
        let query = after().query_tokens("whitfield").unwrap().unwrap();
        assert_eq!(query.previous, None);
        assert!(query.active.iter().all(|token| new_tokens.contains(token)));
    }

    #[test]
    fn search_tokens_are_keyed_hmacs_not_plain_hashes() {
        use sha2::{Digest, Sha256};
        let tokens = before().index_tokens(["whitfield"]).unwrap();
        let other_key = Vault::with_keys(tenant(), NAMESPACE, ("key-2025", NEW), Vec::new())
            .unwrap()
            .index_tokens(["whitfield"])
            .unwrap();
        let other_tenant =
            Vault::with_keys(Uuid::new_v4(), NAMESPACE, ("key-2025", OLD), Vec::new())
                .unwrap()
                .index_tokens(["whitfield"])
                .unwrap();
        // Same words, same key id: only the secret key, or the tenant, differs.
        assert_eq!(tokens.len(), other_key.len());
        assert!(tokens.iter().all(|token| !other_key.contains(token)), "depends on the secret key");
        assert!(tokens.iter().all(|token| !other_tenant.contains(token)), "depends on the tenant");
        // And no token is a bare digest of a word anyone could compute.
        for gram in ["whi", "whit", "whitf", "whitfield"] {
            let digest = Sha256::digest(gram.as_bytes());
            let hex = hex::encode(digest);
            let b64 = {
                use base64::Engine;
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
            };
            assert!(tokens.iter().all(|token| !token.contains(&hex) && !token.ends_with(&b64)));
        }
        assert!(tokens.iter().all(|token| token.starts_with("bi1.key-2025.")));
    }
}
