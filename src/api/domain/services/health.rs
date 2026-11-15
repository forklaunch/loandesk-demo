use crate::{
    document_store::DocumentStore,
    domain::schemas::{DependencyCheck, HealthResponse},
    error::{AppError, Result},
    persistence::Db,
};

#[derive(Clone)]
pub struct HealthService {
    name: String,
    db: Db,
    store: DocumentStore,
}

/// Version and, when the image was built with one, the commit it came from.
pub fn version() -> String {
    match option_env!("GIT_SHA").filter(|sha| !sha.is_empty()) {
        Some(sha) => format!("{}+{sha}", env!("CARGO_PKG_VERSION")),
        None => env!("CARGO_PKG_VERSION").to_owned(),
    }
}

impl HealthService {
    pub fn new(name: String, db: Db, store: DocumentStore) -> Self {
        Self { name, db, store }
    }

    /// Liveness: the process answers. Deliberately checks nothing else, so a
    /// slow dependency never gets a healthy instance restarted.
    pub fn live(&self) -> HealthResponse {
        HealthResponse {
            service: self.name.clone(),
            status: "ok".into(),
            version: version(),
            checks: Vec::new(),
        }
    }

    /// Readiness: pings the database and the object store, each with a short
    /// timeout, and fails when either is down so the instance stops taking traffic.
    pub async fn ready(&self) -> Result<HealthResponse> {
        let (database, objects) = tokio::join!(self.db.ping(), self.store.ping());
        let label = |ok: bool| if ok { "ok" } else { "unavailable" }.to_owned();
        let checks = vec![
            DependencyCheck { name: "database".into(), status: label(database.is_ok()) },
            DependencyCheck { name: "object_store".into(), status: label(objects.is_ok()) },
        ];
        if database.is_err() || objects.is_err() {
            tracing::warn!(
                event = "not_ready",
                database = database.is_ok(),
                object_store = objects.is_ok(),
                "readiness check failed"
            );
            return Err(AppError::Storage("dependency unavailable".into()));
        }
        Ok(HealthResponse {
            service: self.name.clone(),
            status: "ok".into(),
            version: version(),
            checks,
        })
    }
}
