use crate::{
    document_store::DocumentStore,
    domain::schemas::{DependencyCheck, HealthResponse},
    error::{AppError, Result},
    persistence::Db,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub struct HealthService {
    name: String,
    db: Db,
    store: DocumentStore,
    /// Set when shutdown starts: readiness then fails, so the load balancer
    /// stops sending new requests while the in-flight ones finish.
    draining: Arc<AtomicBool>,
}

/// Readiness from its three inputs. Not ready while draining or while any
/// dependency is unreachable.
fn readiness(draining: bool, database: bool, object_store: bool) -> (bool, Vec<DependencyCheck>) {
    let label = |ok: bool| if ok { "ok" } else { "unavailable" }.to_owned();
    let checks = vec![
        DependencyCheck { name: "database".into(), status: label(database) },
        DependencyCheck { name: "object_store".into(), status: label(object_store) },
    ];
    (!draining && database && object_store, checks)
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
        Self { name, db, store, draining: Arc::new(AtomicBool::new(false)) }
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

    /// Called once, when the shutdown signal arrives.
    pub fn begin_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    /// Readiness: pings the database and the object store, each with a short
    /// timeout, and fails when either is down, or once shutdown has started,
    /// so the instance stops taking traffic.
    pub async fn ready(&self) -> Result<HealthResponse> {
        if self.draining.load(Ordering::SeqCst) {
            return Err(AppError::Storage("shutting down".into()));
        }
        let (database, objects) = tokio::join!(self.db.ping(), self.store.ping());
        let (ready, checks) = readiness(false, database.is_ok(), objects.is_ok());
        if !ready {
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

#[cfg(test)]
mod tests {
    use super::readiness;

    #[test]
    fn ready_only_when_every_dependency_answers_and_shutdown_has_not_started() {
        assert!(readiness(false, true, true).0);
        assert!(!readiness(false, false, true).0, "database down");
        assert!(!readiness(false, true, false).0, "object store down");
        assert!(!readiness(true, true, true).0, "draining");
        let (_, checks) = readiness(false, false, true);
        assert_eq!(checks[0].status, "unavailable");
        assert_eq!(checks[1].status, "ok");
    }
}
