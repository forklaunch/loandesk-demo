//! Connection policy and trusted tenant configuration; never derives identity from request input.
use crate::error::{AppError, Result, storage};
use forklaunch_core::{
    config,
    persistence::{PostgresConnectionPolicy, connect_migration_postgres},
};
use sqlx::Connection;
use uuid::Uuid;

pub struct DatabaseConfig {
    pub url: String,
    pub tenant: Uuid,
    pub policy: PostgresConnectionPolicy,
    pub statement_timeout_ms: u32,
}

/// Plaintext database connections are an explicit local-development exception.
/// The framework refuses a remote host without verified TLS unless it is set,
/// and `Settings::validate` refuses to start in production when it is.
pub fn plaintext_development_allowed() -> Result<bool> {
    let allow: String = config::required("DATABASE_ALLOW_PLAINTEXT_DEVELOPMENT")?;
    match allow.as_str() {
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

/// Applies the versioned migrations with the schema-owner role, adopting any
/// existing rows into the configured tenant, then re-encrypts values an
/// earlier version stored in clear (see `persistence::legacy`).
pub async fn migrate() -> Result<()> {
    let url: String = config::required("MIGRATION_DATABASE_URL")?;
    let tenant = tenant()?;
    let pool = connect_migration_postgres(&url, policy()?)
        .await
        .map_err(storage("connect for migration"))?;
    let result = async {
        let mut connection =
            pool.acquire().await.map_err(storage("acquire migration connection"))?;
        sqlx::query("SELECT set_config('forklaunch.tenant_id', $1, false)")
            .bind(tenant.to_string())
            .execute(&mut *connection)
            .await
            .map_err(storage("bind migration tenant"))?;
        sqlx::migrate!("./migrations")
            .run(&mut *connection)
            .await
            .map_err(storage("apply migrations"))?;
        let before = super::legacy::pending(&mut connection).await?;
        if before.total() > 0 {
            let mut tx = connection.begin().await.map_err(storage("begin legacy migration"))?;
            super::legacy::reencrypt(&mut tx, tenant).await?;
            tx.commit().await.map_err(storage("commit legacy migration"))?;
        }
        let after = super::legacy::pending(&mut connection).await?;
        tracing::info!(
            event = "migration_complete",
            legacy_rows_before = before.total(),
            legacy_rows_after = after.total(),
            "database migrated"
        );
        if after.total() > 0 {
            return Err(AppError::Storage("legacy plaintext rows remain after migration".into()));
        }
        Ok(())
    }
    .await;
    pool.close().await;
    result
}
