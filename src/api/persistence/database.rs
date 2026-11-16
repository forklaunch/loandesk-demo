//! Connection policy and trusted tenant configuration; never derives identity from request input.
use crate::error::{AppError, Result, storage};
use forklaunch_core::{
    config,
    persistence::{PostgresConnectionPolicy, connect_migration_postgres},
};
use uuid::Uuid;

pub struct DatabaseConfig {
    pub url: String,
    pub tenant: Uuid,
    pub policy: PostgresConnectionPolicy,
    pub statement_timeout_ms: u32,
}

/// Plaintext database connections are an explicit local-development exception.
/// The framework refuses a remote host without verified TLS unless it is set,
/// and `Settings::validate` refuses to start in production, or as a hosted
/// demo, when it is. Unset means `false`: a deployment that says nothing gets
/// verified TLS.
pub fn plaintext_development_allowed() -> Result<bool> {
    let allow: Option<String> = config::optional("DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT")?;
    match allow.as_deref().unwrap_or("false") {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(AppError::InvalidConfig(
            "DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT must be true or false".into(),
        )),
    }
}
fn policy() -> Result<PostgresConnectionPolicy> {
    Ok(PostgresConnectionPolicy {
        allow_plaintext_development: plaintext_development_allowed()?,
        max_connections: config::optional("DATABASE_MAX_CONNECTIONS")?.unwrap_or(10),
    })
}
fn tenant() -> Result<Uuid> {
    let tenant: Uuid = config::required("TENANT_ID")?;
    if tenant.is_nil() {
        return Err(AppError::InvalidConfig("TENANT_ID must be a trusted non-nil UUID".into()));
    }
    Ok(tenant)
}
impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: config::required("DATABASE_URL")?,
            tenant: tenant()?,
            policy: policy()?,
            statement_timeout_ms: config::optional("DATABASE_STATEMENT_TIMEOUT_MS")?
                .unwrap_or(10_000),
        })
    }
}

/// A pool for the schema-owner role (`MIGRATION_DATABASE_URL`) and the tenant
/// it works on. Used by `--migrate` and `--rotate-keys`; never by the server.
pub async fn migration_pool() -> Result<(sqlx::PgPool, Uuid)> {
    let url: String = config::required("MIGRATION_DATABASE_URL")?;
    let pool = connect_migration_postgres(&url, policy()?)
        .await
        .map_err(storage("connect for migration"))?;
    Ok((pool, tenant()?))
}

/// The last migration that only adds (columns, tables, indexes). Later ones
/// remove what the one-time re-encryption has emptied, so `migrate` runs that
/// re-encryption between the two groups.
pub const LAST_EXPAND_MIGRATION: i64 = 18;

/// Applies the versioned migrations with the schema-owner role, adopting any
/// existing rows into the configured tenant, then re-encrypts values an
/// earlier version stored in clear (see `persistence::legacy`).
pub async fn migrate() -> Result<()> {
    let (pool, tenant) = migration_pool().await?;
    let result = async {
        let mut connection =
            pool.acquire().await.map_err(storage("acquire migration connection"))?;
        sqlx::query("SELECT set_config('forklaunch.tenant_id', $1, false)")
            .bind(tenant.to_string())
            .execute(&mut *connection)
            .await
            .map_err(storage("bind migration tenant"))?;
        // Expand, backfill, contract, in that order and in one command:
        // everything up to the last additive migration, then the one-time
        // re-encryption of values an earlier version stored in clear, and only
        // when none are left the migrations that drop the old columns.
        let migrator = sqlx::migrate!("./migrations");
        migrator
            .run_to(LAST_EXPAND_MIGRATION, &mut *connection)
            .await
            .map_err(storage("apply migrations"))?;
        let (mut before, mut after) = (0, 0);
        if super::legacy::columns_present(&mut connection).await? {
            before = super::legacy::pending(&mut connection).await?.total();
            if before > 0 {
                super::legacy::reencrypt(&mut connection, tenant).await?;
            }
            after = super::legacy::pending(&mut connection).await?.total();
            if after > 0 {
                return Err(AppError::Storage(
                    "legacy plaintext rows remain after migration".into(),
                ));
            }
        }
        migrator.run(&mut *connection).await.map_err(storage("apply migrations"))?;
        // An index build that was interrupted leaves an INVALID index behind,
        // which PostgreSQL keeps but never uses (see migrations 0003 onward).
        let invalid: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_index i JOIN pg_class c ON c.oid = i.indexrelid
             WHERE c.relnamespace = 'public'::regnamespace AND NOT i.indisvalid",
        )
        .fetch_one(&mut *connection)
        .await
        .map_err(storage("check index builds"))?;
        if invalid > 0 {
            return Err(AppError::Storage(
                "an index build did not finish: drop the INVALID index and run --migrate again"
                    .into(),
            ));
        }
        super::rotation::register_active_key(&mut connection, tenant).await?;
        tracing::info!(
            event = "migration_complete",
            legacy_rows_before = before,
            legacy_rows_after = after,
            "database migrated"
        );
        Ok(())
    }
    .await;
    pool.close().await;
    result
}
