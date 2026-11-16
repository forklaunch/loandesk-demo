//! One-time adoption of rows written by the pre-ForkLaunch backend.
//!
//! That version kept owner names, dates of birth, free text and file names in
//! clear, and SSNs under its own AES-256-GCM key. `api --migrate` calls this
//! after the additive schema migrations and outside their transactions: every
//! such value is sealed as a ForkLaunch protected field and the old column is
//! emptied. Rows are read and converted in bounded batches (`BATCH`), and each
//! batch is committed on its own, so no transaction or lock grows with the
//! size of the tables and an interrupted run continues where it stopped (rows
//! already converted no longer match the batch query). `--migrate` fails if
//! anything is left over, and only then drops the old columns (0019). Moving the stored
//! files is a separate step, described in docs/migration.md.
use crate::{
    domain::crypto,
    error::{AppError, Result, storage},
    persistence::Vault,
};
use forklaunch_core::config;
use sqlx::{Connection, PgConnection, Row};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Rows read and converted per round trip.
const BATCH: i64 = 200;
/// The previous backend allowed at most 20 owners per application.
const MAX_OWNERS_PER_BATCH: i64 = BATCH * 20;

pub struct Pending {
    pub owners: i64,
    pub applications: i64,
    pub documents: i64,
}
impl Pending {
    pub fn total(&self) -> i64 {
        self.owners + self.applications + self.documents
    }
}

/// Whether the pre-ForkLaunch plaintext columns still exist. They are dropped
/// by migration 0019 once this module has emptied them; after that there is
/// nothing left for it to do.
pub async fn columns_present(connection: &mut PgConnection) -> Result<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns
                         WHERE table_schema = 'public' AND table_name = 'owners'
                           AND column_name = 'ssn_encrypted')",
    )
    .fetch_one(&mut *connection)
    .await
    .map_err(storage("look for legacy columns"))
}

pub async fn pending(connection: &mut PgConnection) -> Result<Pending> {
    let row = sqlx::query(
        "SELECT (SELECT count(*) FROM owners WHERE ssn_ciphertext IS NULL OR full_name IS NOT NULL
                    OR ssn_encrypted IS NOT NULL OR date_of_birth IS NOT NULL) AS owners,
                (SELECT count(*) FROM applications WHERE purpose IS NOT NULL OR decision_note IS NOT NULL) AS applications,
                (SELECT count(*) FROM documents WHERE filename IS NOT NULL) AS documents",
    )
    .fetch_one(&mut *connection)
    .await
    .map_err(storage("count legacy rows"))?;
    Ok(Pending {
        owners: row.get("owners"),
        applications: row.get("applications"),
        documents: row.get("documents"),
    })
}

struct LegacyOwner {
    id: Uuid,
    application: Uuid,
    name: String,
    ssn_encrypted: Vec<u8>,
    date_of_birth: String,
}

fn legacy_owner(row: &sqlx::postgres::PgRow) -> LegacyOwner {
    LegacyOwner {
        id: row.get("id"),
        application: row.get("application_id"),
        name: row.get("full_name"),
        ssn_encrypted: row.get("ssn_encrypted"),
        date_of_birth: row.get("date_of_birth"),
    }
}

/// Reads the previous backend's SSN ciphertext. Built on first use, so a
/// database without legacy owners does not need the old key.
struct LegacyCipher(Option<crypto::Cipher>);
impl LegacyCipher {
    fn decrypt(&mut self, ciphertext: &[u8]) -> Result<String> {
        if self.0.is_none() {
            // The previous backend's DATA_ENCRYPTION_KEY, needed once to read its SSNs.
            let key: String = config::required("LEGACY_DATA_ENCRYPTION_KEY")?;
            self.0 = Some(crypto::Cipher::from_base64(&key).map_err(|_| {
                AppError::InvalidConfig(
                    "LEGACY_DATA_ENCRYPTION_KEY must be the previous base64 key".into(),
                )
            })?);
        }
        self.0.as_ref().expect("cipher was just built").decrypt(ciphertext).map_err(|_| {
            AppError::Storage(
                "a legacy SSN could not be decrypted with LEGACY_DATA_ENCRYPTION_KEY".into(),
            )
        })
    }
}

async fn seal_owner(
    connection: &mut PgConnection,
    vault: &Vault,
    cipher: &mut LegacyCipher,
    owner: &LegacyOwner,
) -> Result<()> {
    let members = &vault.members;
    let ssn = cipher.decrypt(&owner.ssn_encrypted)?;
    sqlx::query(
        "UPDATE owners SET full_name_ciphertext = $2, ssn_ciphertext = $3, date_of_birth_ciphertext = $4,
                full_name = NULL, ssn_encrypted = NULL, ssn_last4 = NULL, date_of_birth = NULL
         WHERE id = $1",
    )
    .bind(owner.id)
    .bind(vault.seal(&members.owner_name, owner.id, &owner.name)?.ciphertext())
    .bind(vault.seal(&members.owner_ssn, owner.id, &ssn)?.ciphertext())
    .bind(vault.seal(&members.owner_dob, owner.id, &owner.date_of_birth)?.ciphertext())
    .execute(&mut *connection)
    .await
    .map_err(storage("re-encrypt owner"))?;
    Ok(())
}

pub async fn reencrypt(connection: &mut PgConnection, tenant: Uuid) -> Result<()> {
    let vault = Vault::from_env(tenant)?;
    let members = vault.members.clone();
    let mut cipher = LegacyCipher(None);
    let (mut owners_done, mut applications_done, mut documents_done) = (0_u64, 0_u64, 0_u64);

    // Applications, one bounded batch at a time, each with its own owners: the
    // search tokens of an application are built from its purpose and its
    // owners' names, so both are converted together. A converted row no longer
    // matches the WHERE clause, so every round reads the next batch.
    loop {
        let mut tx = connection.begin().await.map_err(storage("begin legacy batch"))?;
        let applications = sqlx::query(
            "SELECT id, purpose, decision_note FROM applications
             WHERE purpose IS NOT NULL OR decision_note IS NOT NULL
             ORDER BY id LIMIT $1",
        )
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read legacy applications"))?;
        if applications.is_empty() {
            break;
        }
        let ids: Vec<Uuid> = applications.iter().map(|row| row.get("id")).collect();
        let owners = sqlx::query(
            "SELECT id, application_id, full_name, ssn_encrypted, date_of_birth::text AS date_of_birth
             FROM owners WHERE application_id = ANY($1) AND ssn_ciphertext IS NULL
             ORDER BY application_id, id LIMIT $2",
        )
        .bind(&ids)
        .bind(MAX_OWNERS_PER_BATCH + 1)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read legacy owners"))?;
        if owners.len() > usize::try_from(MAX_OWNERS_PER_BATCH).unwrap_or(usize::MAX) {
            return Err(AppError::Storage(
                "a batch of legacy applications has more owners than the previous version allowed"
                    .into(),
            ));
        }
        let mut names: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
        for row in &owners {
            let owner = legacy_owner(row);
            seal_owner(&mut tx, &vault, &mut cipher, &owner).await?;
            names.entry(owner.application).or_default().push(owner.name);
            owners_done += 1;
        }
        for row in &applications {
            let id: Uuid = row.get("id");
            let purpose: Option<String> = row.get("purpose");
            let note: Option<String> = row.get("decision_note");
            let owner_names = names.remove(&id).unwrap_or_default();
            let tokens = vault.index_tokens(
                purpose.iter().map(String::as_str).chain(owner_names.iter().map(String::as_str)),
            )?;
            let sealed_purpose = purpose
                .as_deref()
                .map(|text| vault.seal(&members.purpose, id, text))
                .transpose()?;
            let sealed_note = note
                .as_deref()
                .map(|text| vault.seal(&members.decision_note, id, text))
                .transpose()?;
            sqlx::query(
                "UPDATE applications SET purpose_ciphertext = COALESCE($2, purpose_ciphertext),
                        decision_note_ciphertext = COALESCE($3, decision_note_ciphertext),
                        search_tokens = $4, purpose = NULL, decision_note = NULL
                 WHERE id = $1",
            )
            .bind(id)
            .bind(sealed_purpose.as_ref().map(|field| field.ciphertext().to_owned()))
            .bind(sealed_note.as_ref().map(|field| field.ciphertext().to_owned()))
            .bind(&tokens)
            .execute(&mut *tx)
            .await
            .map_err(storage("re-encrypt application"))?;
            applications_done += 1;
        }
        tx.commit().await.map_err(storage("commit legacy batch"))?;
    }

    // Owners whose application was already converted (an interrupted earlier run).
    loop {
        let mut tx = connection.begin().await.map_err(storage("begin legacy batch"))?;
        let owners = sqlx::query(
            "SELECT id, application_id, full_name, ssn_encrypted, date_of_birth::text AS date_of_birth
             FROM owners WHERE ssn_ciphertext IS NULL ORDER BY id LIMIT $1",
        )
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read legacy owners"))?;
        if owners.is_empty() {
            break;
        }
        for row in &owners {
            seal_owner(&mut tx, &vault, &mut cipher, &legacy_owner(row)).await?;
            owners_done += 1;
        }
        tx.commit().await.map_err(storage("commit legacy batch"))?;
    }

    loop {
        let mut tx = connection.begin().await.map_err(storage("begin legacy batch"))?;
        let documents = sqlx::query(
            "SELECT id, filename FROM documents WHERE filename IS NOT NULL ORDER BY id LIMIT $1",
        )
        .bind(BATCH)
        .fetch_all(&mut *tx)
        .await
        .map_err(storage("read legacy documents"))?;
        if documents.is_empty() {
            break;
        }
        for row in &documents {
            let id: Uuid = row.get("id");
            let label: String = row.get("filename");
            sqlx::query(
                "UPDATE documents SET filename_ciphertext = $2, filename = NULL WHERE id = $1",
            )
            .bind(id)
            .bind(vault.seal(&members.filename, id, &label)?.ciphertext())
            .execute(&mut *tx)
            .await
            .map_err(storage("re-encrypt document name"))?;
            documents_done += 1;
        }
        tx.commit().await.map_err(storage("commit legacy batch"))?;
    }
    tracing::info!(
        event = "legacy_rows_reencrypted",
        owners = owners_done,
        applications = applications_done,
        documents = documents_done,
        batch_size = BATCH,
        "legacy plaintext re-encrypted"
    );
    Ok(())
}
