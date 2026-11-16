//! Database access shared by every service.
//!
//! SQL stays in the services that own it. This type owns the things that must
//! be the same everywhere: every statement runs in a transaction bound to the
//! trusted tenant (row-level security) with a statement timeout; classified
//! values are sealed before they reach SQL and only come back out through an
//! audited reveal; audit rows are written in the same transaction as the
//! change they describe, so neither can happen without the other.
use crate::{
    api::transport,
    dependency_metrics::{POSTGRESQL, measured},
    error::{AppError, Result, storage},
    persistence::{
        database::DatabaseConfig,
        entities::{StoredApplication, StoredDocument, StoredOwner, StoredUserMfa},
    },
};
use forklaunch_core::{
    common::FrameworkResult,
    config,
    encryption::{EnvelopeError, EnvelopeKey, EnvelopeProfile},
    persistence::{
        TenantContext, begin_tenant_transaction,
        compliance::ComplianceLevel,
        connect_runtime_postgres,
        protected::{
            AsyncComplianceAudit, ComplianceAccess, DisclosurePolicy, IndexNormalization,
            ProtectedContext, ProtectedField, ProtectedMember, ProtectedQuery,
            migration::{MigrationAudit, MigrationKind, MigrationRequest},
        },
    },
};
use sqlx::{PgPool, Postgres, Transaction};
use std::{
    collections::BTreeSet,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub type Tx<'a> = Transaction<'a, Postgres>;

/// Encryption bindings of every classified column, resolved once from the
/// stored models so a typo cannot silently use the wrong purpose.
#[derive(Clone)]
pub struct Members {
    pub owner_name: ProtectedMember,
    pub owner_ssn: ProtectedMember,
    pub owner_dob: ProtectedMember,
    pub purpose: ProtectedMember,
    pub decision_note: ProtectedMember,
    pub filename: ProtectedMember,
    pub mfa_secret: ProtectedMember,
    search_token: ProtectedMember,
}
impl Members {
    fn resolve() -> FrameworkResult<Self> {
        Ok(Self {
            owner_name: StoredOwner::protected_full_name_ciphertext()?,
            owner_ssn: StoredOwner::protected_ssn_ciphertext()?
                .with_disclosure(DisclosurePolicy::Last4),
            owner_dob: StoredOwner::protected_date_of_birth_ciphertext()?,
            purpose: StoredApplication::protected_purpose_ciphertext()?,
            decision_note: StoredApplication::protected_decision_note_ciphertext()?,
            filename: StoredDocument::protected_filename_ciphertext()?,
            mfa_secret: StoredUserMfa::protected_secret_ciphertext()?,
            search_token: ProtectedMember::new(
                "Application",
                "search_tokens",
                "application-search-token-v1",
                ComplianceLevel::Pii,
            )?
            .with_equality_index(IndexNormalization::TrimLowercase),
        })
    }
}

/// Field encryption for one tenant: the key, and the binding of every
/// classified column. Holds no database connection.
#[derive(Clone)]
pub struct Vault {
    tenant: Uuid,
    key: Arc<EnvelopeKey>,
    /// `ENCRYPTION_KEY_ID`: the key every new value and search token is written under.
    active_key_id: Arc<str>,
    pub members: Arc<Members>,
}

/// The search tokens a query must match. Rows written before a key rotation
/// carry tokens under the previous key until `api --rotate-keys` has rebuilt
/// them, so while a previous key is configured a search matches either set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTokens {
    pub active: Vec<String>,
    pub previous: Option<Vec<String>>,
}

/// The id of the key a stored protected value was sealed under: the `<id>` of
/// `v3.<id>.<nonce>.<ciphertext>`, also when that envelope is wrapped in a
/// `pd1.` disclosure record (SSNs). `None` for anything else.
pub fn stored_key_id(stored: &str) -> Option<String> {
    use base64::Engine;
    let inner = match stored.strip_prefix("pd1.") {
        Some(encoded) => {
            let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded).ok()?;
            let record: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
            record.get("ciphertext")?.as_str()?.to_owned()
        }
        None => stored.to_owned(),
    };
    let mut parts = inner.split('.');
    matches!(parts.next()?, "v2" | "v3").then(|| parts.next().map(str::to_owned))?
}

/// The id of the key a search token was derived under (`bi1.<id>.<mac>`).
pub fn token_key_id(token: &str) -> Option<&str> {
    let mut parts = token.split('.');
    (parts.next()? == "bi1").then(|| parts.next())?
}

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
    context: TenantContext,
    vault: Vault,
    pub members: Arc<Members>,
    statement_timeout_ms: u32,
}

fn unavailable(operation: &'static str) -> AppError {
    AppError::Storage(operation.to_owned())
}

impl Vault {
    pub fn from_env(tenant: Uuid) -> Result<Self> {
        let namespace: String = config::required("STORAGE_ENCRYPTION_NAMESPACE")?;
        let profile =
            EnvelopeProfile::new(&namespace).map_err(|_| unavailable("encryption profile"))?;
        // Keys come from the environment by id (ENCRYPTION_KEY_ID, ENCRYPTION_KEY_HEX,
        // ENCRYPTION_PREVIOUS_KEYS_JSON): rotating one is a configuration change.
        let key = EnvelopeKey::from_env(profile).map_err(|error| match error {
            EnvelopeError::Configuration(error) => error.into(),
            _ => unavailable("encryption key"),
        })?;
        let active_key_id: String = config::required("ENCRYPTION_KEY_ID")?;
        Ok(Self {
            tenant,
            key: Arc::new(key),
            active_key_id: active_key_id.into(),
            members: Arc::new(Members::resolve()?),
        })
    }

    /// A vault over explicit key material instead of the environment: the
    /// active key and the previous keys that stay readable. Used by the
    /// rotation tests; processes use [`Vault::from_env`].
    pub fn with_keys(
        tenant: Uuid,
        namespace: &str,
        active: (&str, [u8; 32]),
        previous: Vec<(String, [u8; 32])>,
    ) -> Result<Self> {
        let profile =
            EnvelopeProfile::new(namespace).map_err(|_| unavailable("encryption profile"))?;
        let key =
            EnvelopeKey::with_rotation(profile, active.0.to_owned(), active.1, previous, None)
                .map_err(|_| unavailable("encryption key"))?;
        Ok(Self {
            tenant,
            key: Arc::new(key),
            active_key_id: active.0.into(),
            members: Arc::new(Members::resolve()?),
        })
    }

    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    /// Whether a stored value is already sealed under the active key.
    pub fn is_current(&self, stored: &str) -> bool {
        stored_key_id(stored).as_deref() == Some(self.active_key_id())
    }

    /// Re-seals one stored value under the active key, without changing what
    /// it decrypts to or its disclosure projection (an SSN keeps its last
    /// four). The old value is authenticated with the key it names, which must
    /// still be configured as a previous key. The read and the rewrite are
    /// both acknowledged by `log` before the new ciphertext is returned.
    pub async fn rekey(
        &self,
        member: &ProtectedMember,
        resource: Uuid,
        stored: String,
        operation: Uuid,
        log: &(impl AsyncComplianceAudit + MigrationAudit),
    ) -> Result<String> {
        let context = self.context_for(resource)?;
        let field = ProtectedField::from_ciphertext(member.clone(), stored)
            .map_err(storage("load protected field"))?;
        let plan = field
            .prepare_migration(
                MigrationRequest {
                    source_key: &self.key,
                    source_context: context,
                    target_key: &self.key,
                    target_context: context,
                    target_member: member.clone(),
                    operation_id: operation,
                    kind: MigrationKind::Rotate,
                    actor: "system:key-rotation",
                    reason: "rotate-encryption-key",
                },
                log,
                log,
            )
            .await
            .map_err(storage("re-seal protected field"))?;
        Ok(plan.replacement().ciphertext().to_owned())
    }

    fn context_for(&self, resource: Uuid) -> Result<ProtectedContext> {
        ProtectedContext::new(self.tenant, resource).map_err(|_| unavailable("protected context"))
    }

    /// Encrypts one classified value for the row `resource`.
    pub fn seal(
        &self,
        member: &ProtectedMember,
        resource: Uuid,
        plaintext: &str,
    ) -> Result<ProtectedField> {
        ProtectedField::seal(
            &self.key,
            self.context_for(resource)?,
            member.clone(),
            plaintext.as_bytes(),
        )
        .map_err(storage("seal protected field"))
    }

    /// Decrypts one classified value. The access is recorded in `disclosures`,
    /// which the caller must persist in its transaction before responding.
    pub async fn reveal(
        &self,
        member: &ProtectedMember,
        resource: Uuid,
        ciphertext: String,
        actor: Uuid,
        reason: &'static str,
        disclosures: &dyn AsyncComplianceAudit,
    ) -> Result<String> {
        let field = ProtectedField::from_ciphertext(member.clone(), ciphertext)
            .map_err(storage("load protected field"))?;
        let plaintext = field
            .reveal_async(
                &self.key,
                self.context_for(resource)?,
                &actor.to_string(),
                reason,
                disclosures,
            )
            .await
            .map_err(storage("reveal protected field"))?;
        String::from_utf8(plaintext.to_vec())
            .map_err(|_| unavailable("protected field is not text"))
    }

    /// Last four digits of a sealed SSN, from its authenticated disclosure
    /// projection: the full number is not decrypted.
    pub fn ssn_last4(&self, owner: Uuid, ciphertext: String) -> Result<String> {
        let field = ProtectedField::from_ciphertext(self.members.owner_ssn.clone(), ciphertext)
            .map_err(storage("load protected field"))?;
        let disclosed = field
            .disclosed(&self.key, self.context_for(owner)?)
            .map_err(storage("disclose ssn last four"))?;
        Ok(disclosed.as_str().chars().filter(char::is_ascii_digit).collect())
    }

    /// HMAC-SHA-256 tokens of one word: under the active key first, then
    /// under each previous key that is still configured. The HMAC key is
    /// derived (HKDF) from the secret encryption key, the tenant and the
    /// field's purpose, so a token cannot be computed, or a name guessed and
    /// confirmed, without that secret.
    fn tokens(&self, word: &str) -> Result<Vec<String>> {
        let tokens = self
            .members
            .search_token
            .query(&self.key, self.tenant, ProtectedQuery::Equal(word))
            .map_err(storage("derive search token"))?
            .values()
            .to_vec();
        if tokens.is_empty() {
            return Err(unavailable("derive search token"));
        }
        Ok(tokens)
    }

    fn token(&self, word: &str) -> Result<String> {
        Ok(self.tokens(word)?.swap_remove(0))
    }

    /// Keyed tokens stored beside encrypted text so it stays searchable by
    /// word or word prefix (3+ letters) without being readable.
    pub fn index_tokens<'a>(
        &self,
        texts: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<String>> {
        let mut grams = BTreeSet::new();
        for text in texts {
            for word in words(text) {
                let chars: Vec<char> = word.chars().collect();
                if chars.len() < 3 {
                    grams.insert(word);
                    continue;
                }
                for end in 3..=chars.len().min(MAX_TOKEN_CHARS) {
                    grams.insert(chars[..end].iter().collect::<String>());
                }
            }
        }
        grams.iter().take(2000).map(|gram| self.token(gram)).collect()
    }

    /// Tokens every match must contain, or `None` when the query has no usable word.
    pub fn query_tokens(&self, query: &str) -> Result<Option<QueryTokens>> {
        let words = words(query);
        if words.is_empty() || words.len() > 8 {
            return Ok(None);
        }
        let mut active = Vec::with_capacity(words.len());
        let mut previous = Vec::with_capacity(words.len());
        for word in &words {
            let mut tokens =
                self.tokens(&word.chars().take(MAX_TOKEN_CHARS).collect::<String>())?.into_iter();
            active.extend(tokens.next());
            previous.extend(tokens.next());
        }
        let previous = (previous.len() == words.len()).then_some(previous);
        Ok(Some(QueryTokens { active, previous }))
    }
}

impl Db {
    pub async fn connect(config: DatabaseConfig) -> Result<Self> {
        let vault = Vault::from_env(config.tenant)?;
        let context = TenantContext::new(config.tenant.to_string())
            .map_err(|_| unavailable("tenant context"))?;
        // Rejects remote plaintext connections and privileged roles; pool acquire times out.
        let pool = connect_runtime_postgres(&config.url, config.policy)
            .await
            .map_err(storage("connect to PostgreSQL"))?;
        let db = Self {
            pool,
            context,
            members: vault.members.clone(),
            vault,
            statement_timeout_ms: config.statement_timeout_ms,
        };
        // Migration ownership is explicit; startup never silently creates
        // storage. The schema may still be on its way (the worker's task
        // migrates), so wait for it before the first query that needs it.
        if let Err(error) = super::database::wait_for_schema(&db.pool).await {
            db.close().await;
            return Err(error);
        }
        if let Err(error) = db.ping().await {
            db.close().await;
            return Err(error);
        }
        Ok(db)
    }

    pub fn tenant(&self) -> Uuid {
        self.vault.tenant
    }
    pub fn vault(&self) -> &Vault {
        &self.vault
    }
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// A transaction scoped to the tenant, with a statement timeout so one slow
    /// query cannot hold a connection indefinitely.
    pub async fn tx(&self) -> Result<Tx<'_>> {
        let mut tx =
            measured(POSTGRESQL, "begin", begin_tenant_transaction(&self.pool, &self.context))
                .await
                .map_err(storage("begin transaction"))?;
        sqlx::query("SELECT set_config('statement_timeout', $1, true)")
            .bind(self.statement_timeout_ms.to_string())
            .execute(&mut *tx)
            .await
            .map_err(storage("set statement timeout"))?;
        Ok(tx)
    }

    /// Readiness probe: one cheap query against a migrated table, bounded by a short timeout.
    pub async fn ping(&self) -> Result<()> {
        let check = async {
            let mut tx = self.tx().await?;
            sqlx::query("SELECT 1 FROM users LIMIT 1")
                .execute(&mut *tx)
                .await
                .map_err(storage("readiness query"))?;
            tx.commit().await.map_err(storage("readiness query"))
        };
        measured(POSTGRESQL, "ping", async {
            tokio::time::timeout(std::time::Duration::from_secs(2), check)
                .await
                .map_err(|_| unavailable("readiness query timed out"))?
        })
        .await
    }

    pub fn seal(
        &self,
        member: &ProtectedMember,
        resource: Uuid,
        plaintext: &str,
    ) -> Result<ProtectedField> {
        self.vault.seal(member, resource, plaintext)
    }
    pub async fn reveal(
        &self,
        member: &ProtectedMember,
        resource: Uuid,
        ciphertext: String,
        actor: Uuid,
        reason: &'static str,
        disclosures: &dyn AsyncComplianceAudit,
    ) -> Result<String> {
        self.vault.reveal(member, resource, ciphertext, actor, reason, disclosures).await
    }
    pub fn ssn_last4(&self, owner: Uuid, ciphertext: String) -> Result<String> {
        self.vault.ssn_last4(owner, ciphertext)
    }
    pub fn index_tokens<'a>(
        &self,
        texts: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<String>> {
        self.vault.index_tokens(texts)
    }
    pub fn query_tokens(&self, query: &str) -> Result<Option<QueryTokens>> {
        self.vault.query_tokens(query)
    }

    /// Appends to the audit trail inside the caller's transaction.
    pub async fn audit(
        &self,
        tx: &mut Tx<'_>,
        user: Option<Uuid>,
        action: &str,
        entity: Option<Uuid>,
        detail: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user)
        .bind(action)
        .bind(entity)
        .bind(detail)
        .bind(transport::request_id())
        .execute(&mut **tx)
        .await
        .map_err(storage("write audit entry"))?;
        Ok(())
    }
}

const MAX_TOKEN_CHARS: usize = 12;

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Protected-field reads made while handling one request. The framework hands
/// every decryption to this sink; `persist` writes them to the audit trail in
/// the request's transaction. A response is only built after that commit, so a
/// value cannot be disclosed without its audit entry.
#[derive(Default)]
pub struct Disclosures {
    events: Mutex<Vec<(Uuid, String)>>,
}
impl Disclosures {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn persist(self, tx: &mut Tx<'_>, actor: Uuid) -> Result<()> {
        let events = std::mem::take(&mut *self.events.lock().expect("disclosure lock"));
        if events.is_empty() {
            return Ok(());
        }
        let (resources, fields): (Vec<Uuid>, Vec<String>) = events.into_iter().unzip();
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, entity_id, detail, request_id)
             SELECT $1, 'protected_field_read', resource, field, $4
             FROM unnest($2::uuid[], $3::text[]) AS read(resource, field)",
        )
        .bind(actor)
        .bind(resources)
        .bind(fields)
        .bind(transport::request_id())
        .execute(&mut **tx)
        .await
        .map_err(storage("write disclosure audit"))?;
        Ok(())
    }
}
impl Drop for Disclosures {
    fn drop(&mut self) {
        let pending = self.events.lock().map(|events| events.len()).unwrap_or(0);
        if pending > 0 && !std::thread::panicking() {
            // Reached only when a request failed after decrypting: nothing was returned.
            tracing::warn!(
                event = "disclosure_not_returned",
                pending,
                request_id = %transport::request_id(),
                "protected fields were decrypted but the request failed before responding"
            );
        }
    }
}
impl AsyncComplianceAudit for Disclosures {
    fn record<'a>(
        &'a self,
        event: ComplianceAccess<'a>,
    ) -> Pin<Box<dyn Future<Output = FrameworkResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.events.lock().expect("disclosure lock").push((
                event.resource,
                format!("{}.{}: {}", event.entity, event.field, event.reason),
            ));
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::words;

    #[test]
    fn search_words_are_lowercased_and_split_on_punctuation() {
        assert_eq!(words("Eleanor  Whitfield-O'Neil"), ["eleanor", "whitfield", "o", "neil"]);
        assert!(words(" - ").is_empty());
    }
}
