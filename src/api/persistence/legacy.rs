//! One-time adoption of rows written by the pre-ForkLaunch backend.
//!
//! That version kept owner names, dates of birth, free text and file names in
//! clear, and SSNs under its own AES-256-GCM key. `api --migrate` calls this
//! after the schema migrations: every such value is sealed as a ForkLaunch
//! protected field and the old column is emptied, in one transaction. It is
//! idempotent (rows already converted are skipped) and `--migrate` fails if
//! anything is left over. Stored files are moved by the separate
//! `scripts/migrate-legacy-files.sh` step described in docs/migration.
use crate::{
    domain::crypto,
    error::{AppError, Result, storage},
    persistence::Vault,
};
use forklaunch_core::config;
use sqlx::{PgConnection, Row};
use uuid::Uuid;

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

pub async fn reencrypt(connection: &mut PgConnection, tenant: Uuid) -> Result<()> {
    let vault = Vault::from_env(tenant)?;
    let members = vault.members.clone();

    let owners = sqlx::query(
        "SELECT id, application_id, full_name, ssn_encrypted, date_of_birth::text AS date_of_birth
         FROM owners WHERE ssn_ciphertext IS NULL",
    )
    .fetch_all(&mut *connection)
    .await
    .map_err(storage("read legacy owners"))?;
    let cipher = if owners.is_empty() {
        None
    } else {
        // The previous backend's DATA_ENCRYPTION_KEY, needed once to read its SSNs.
        let key: String = config::required("LEGACY_DATA_ENCRYPTION_KEY")?;
        Some(crypto::Cipher::from_base64(&key).map_err(|_| {
            AppError::InvalidConfig(
                "LEGACY_DATA_ENCRYPTION_KEY must be the previous base64 key".into(),
            )
        })?)
    };
    let mut names: std::collections::BTreeMap<Uuid, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in &owners {
        let id: Uuid = row.get("id");
        let name: String = row.get("full_name");
        let ssn = cipher
            .as_ref()
            .expect("cipher exists when owners do")
            .decrypt(&row.get::<Vec<u8>, _>("ssn_encrypted"))
            .map_err(|_| {
                AppError::Storage(
                    "a legacy SSN could not be decrypted with LEGACY_DATA_ENCRYPTION_KEY".into(),
                )
            })?;
        sqlx::query(
            "UPDATE owners SET full_name_ciphertext = $2, ssn_ciphertext = $3, date_of_birth_ciphertext = $4,
                    full_name = NULL, ssn_encrypted = NULL, ssn_last4 = NULL, date_of_birth = NULL
             WHERE id = $1",
        )
        .bind(id)
        .bind(vault.seal(&members.owner_name, id, &name)?.ciphertext())
        .bind(vault.seal(&members.owner_ssn, id, &ssn)?.ciphertext())
        .bind(vault.seal(&members.owner_dob, id, &row.get::<String, _>("date_of_birth"))?.ciphertext())
        .execute(&mut *connection)
        .await
        .map_err(storage("re-encrypt owner"))?;
        names.entry(row.get("application_id")).or_default().push(name);
    }

    let applications = sqlx::query(
        "SELECT id, purpose, decision_note FROM applications WHERE purpose IS NOT NULL OR decision_note IS NOT NULL",
    )
    .fetch_all(&mut *connection)
    .await
    .map_err(storage("read legacy applications"))?;
    for row in &applications {
        let id: Uuid = row.get("id");
        let purpose: Option<String> = row.get("purpose");
        let note: Option<String> = row.get("decision_note");
        let owner_names = names.remove(&id).unwrap_or_default();
        let tokens = vault.index_tokens(
            purpose.iter().map(String::as_str).chain(owner_names.iter().map(String::as_str)),
        )?;
        let sealed_purpose =
            purpose.as_deref().map(|text| vault.seal(&members.purpose, id, text)).transpose()?;
        let sealed_note =
            note.as_deref().map(|text| vault.seal(&members.decision_note, id, text)).transpose()?;
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
        .execute(&mut *connection)
        .await
        .map_err(storage("re-encrypt application"))?;
    }

    let documents = sqlx::query("SELECT id, filename FROM documents WHERE filename IS NOT NULL")
        .fetch_all(&mut *connection)
        .await
        .map_err(storage("read legacy documents"))?;
    for row in &documents {
        let id: Uuid = row.get("id");
        sqlx::query("UPDATE documents SET filename_ciphertext = $2, filename = NULL WHERE id = $1")
            .bind(id)
            .bind(
                vault.seal(&members.filename, id, &row.get::<String, _>("filename"))?.ciphertext(),
            )
            .execute(&mut *connection)
            .await
            .map_err(storage("re-encrypt document name"))?;
    }
    tracing::info!(
        event = "legacy_rows_reencrypted",
        owners = owners.len(),
        applications = applications.len(),
        documents = documents.len(),
        "legacy plaintext re-encrypted"
    );
    Ok(())
}
