use crate::{
    api::controllers::health::{
        contract::{Bindings, Health, Live, Ready},
        health, live, ready,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings { health: Health::bind(health), ready: Ready::bind(ready), live: Live::bind(live) }
        .mount(router)
}
