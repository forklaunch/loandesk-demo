use crate::{
    api::controllers::health::{
        client_settings,
        contract::{Bindings, ClientSettings, Health, Live, Ready},
        health, live, ready,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings {
        health: Health::bind(health),
        ready: Ready::bind(ready),
        live: Live::bind(live),
        client_settings: ClientSettings::bind(client_settings),
    }
    .mount(router)
}
