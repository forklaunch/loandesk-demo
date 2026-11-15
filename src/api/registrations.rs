use crate::{
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
}

pub async fn register(config: ServiceConfig) -> Result<AppState> {
    config.validate()?;
    let settings = Settings::from_env()?;
    settings.validate(&config, plaintext_development_allowed()?)?;
    let db = Db::connect(DatabaseConfig::from_env()?).await?;
    let store = DocumentStore::from_env(db.tenant()).await?;
    let queue = ExtractionQueue::connect(&db).await?;
    let telemetry = OpenTelemetryCollector::new(config.name.clone());
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
        users: UsersService::new(db.clone()),
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
