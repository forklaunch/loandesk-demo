use crate::{
    api::{
        controllers::browser_errors::{
            contract::{Bindings, Report},
            report,
        },
        policy::SessionCookiePolicy,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings { report: Report::bind_authorized(SessionCookiePolicy, report) }.mount(router)
}
