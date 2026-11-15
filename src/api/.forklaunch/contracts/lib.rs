//! Generated private compiler host; edit controller contracts and domain schemas.
#[path = "../../domain/schemas/mod.rs"]
pub mod schemas;
pub mod domain {
    pub use crate::schemas;
}
forklaunch_core::contract::controller_contract!(
    applications,
    "../../api/controllers/applications.rs"
);
forklaunch_core::contract::controller_contract!(auth, "../../api/controllers/auth.rs");
forklaunch_core::contract::controller_contract!(documents, "../../api/controllers/documents.rs");
forklaunch_core::contract::controller_contract!(health, "../../api/controllers/health.rs");
forklaunch_core::contract::controller_contract!(users, "../../api/controllers/users.rs");

pub fn routes() -> Vec<forklaunch_core::contract::RouteContract> {
    let mut routes = Vec::new();
    routes.extend(applications::contract::registry().routes().iter().cloned());
    routes.extend(auth::contract::registry().routes().iter().cloned());
    routes.extend(documents::contract::registry().routes().iter().cloned());
    routes.extend(health::contract::registry().routes().iter().cloned());
    routes.extend(users::contract::registry().routes().iter().cloned());
    routes
}
pub fn registry() -> forklaunch_core::contract::ContractRegistry {
    let mut registry = forklaunch_core::contract::ContractRegistry::for_service(
        forklaunch_core::contract::ServiceContract::new("api", env!("CARGO_PKG_VERSION")),
    );
    for route in routes() {
        registry.register(route);
    }
    registry
}
pub fn binding_manifest()
-> Result<Vec<forklaunch_core::contract::assembly::BindingDeclaration>, String> {
    let mut declarations = Vec::new();
    declarations.extend(applications::contract::binding_manifest()?);
    declarations.extend(auth::contract::binding_manifest()?);
    declarations.extend(documents::contract::binding_manifest()?);
    declarations.extend(health::contract::binding_manifest()?);
    declarations.extend(users::contract::binding_manifest()?);
    Ok(declarations)
}
