use crate::{
    api::{
        controllers::applications::{
            contract::{
                Bindings, CreateApplication, DecideApplication, ExportApplication, GetApplication,
                ListApplications, RevealSsn, SearchApplications,
            },
            create_application, decide_application, export_application, get_application,
            list_applications, reveal_ssn, search_applications,
        },
        policy::SessionCookiePolicy,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    let policy = SessionCookiePolicy;
    Bindings {
        list_applications: ListApplications::bind_authorized(policy, list_applications),
        search_applications: SearchApplications::bind_authorized(policy, search_applications),
        create_application: CreateApplication::bind_authorized(policy, create_application),
        get_application: GetApplication::bind_authorized(policy, get_application),
        decide_application: DecideApplication::bind_authorized(policy, decide_application),
        reveal_ssn: RevealSsn::bind_authorized(policy, reveal_ssn),
        export_application: ExportApplication::bind_authorized(policy, export_application),
    }
    .mount(router)
}
