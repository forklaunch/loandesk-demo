pub mod applications;
pub mod auth;
pub mod browser_errors;
pub mod documents;
pub mod health;
pub mod users;

use crate::registrations::AppState;
use forklaunch_axum::native::Router;
pub fn router() -> Result<Router<AppState>, String> {
    let router = forklaunch_axum::native::ContractRouter::new(contracts::registry());
    let router = applications::mount(router)?;
    let router = auth::mount(router)?;
    let router = browser_errors::mount(router)?;
    let router = documents::mount(router)?;
    let router = health::mount(router)?;
    let router = users::mount(router)?;
    router.finish()
}
