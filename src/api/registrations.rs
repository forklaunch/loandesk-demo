use crate::{
    api::rate_limit::RateLimiter,
    config::{ServiceConfig, Settings},
    document_store::DocumentStore,
    domain::services::{
        ApplicationsService, AuthService, BrowserErrors, DocumentsService, HealthService,
        UsersService,
    },
    error::Result,
    extraction_queue::ExtractionQueue,
    persistence::{
        Db,
        database::{DatabaseConfig, plaintext_development_allowed},
    },
};
use forklaunch_core::telemetry::OpenTelemetryCollector;
use std::sync::Arc;

/// The three request limits (`api::rate_limit`), shared by every request this instance serves.
pub struct RateLimits {
    pub login: RateLimiter,
    pub requests: RateLimiter,
    pub expensive: RateLimiter,
}

#[derive(Clone)]
pub struct AppState {
    pub config: ServiceConfig,
    pub settings: Settings,
    pub db: Db,
    pub store: DocumentStore,
    pub queue: ExtractionQueue,
    pub health: HealthService,
    pub auth: AuthService,
    pub applications: ApplicationsService,
    pub documents: DocumentsService,
    pub users: UsersService,
    pub browser_errors: BrowserErrors,
    pub telemetry: OpenTelemetryCollector,
    pub limits: Arc<RateLimits>,
}

pub async fn register(config: ServiceConfig) -> Result<AppState> {
    config.validate()?;
    let settings = Settings::from_env()?;
    settings.validate(&config, plaintext_development_allowed()?)?;
    let telemetry = OpenTelemetryCollector::new(config.name.clone());
    // Duration and failure metrics for PostgreSQL and object-store calls.
    crate::dependency_metrics::install(telemetry.clone())
        .map_err(|reason| crate::error::AppError::InvalidConfig(reason.to_owned()))?;
    let db = Db::connect(DatabaseConfig::from_env()?).await?;
    let store = DocumentStore::from_env(db.tenant()).await?;
    let queue = ExtractionQueue::connect(&db).await?;
    let applications = ApplicationsService::new(db.clone());
    Ok(AppState {
        health: HealthService::new(config.name.clone(), db.clone(), store.clone()),
        auth: AuthService::new(db.clone(), settings.clone())?,
        documents: DocumentsService::new(
            db.clone(),
            applications.clone(),
            store.clone(),
            queue.clone(),
        ),
        users: UsersService::new(db.clone(), settings.clone()),
        limits: Arc::new(RateLimits {
            login: RateLimiter::new(settings.rate_limit_login_per_minute),
            requests: RateLimiter::new(settings.rate_limit_requests_per_minute),
            expensive: RateLimiter::new(settings.rate_limit_expensive_per_minute),
        }),
        browser_errors: BrowserErrors::new(),
        applications,
        telemetry,
        queue,
        store,
        db,
        settings,
        config,
    })
}
