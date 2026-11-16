use crate::{
    api::{
        controllers::auth::{
            change_password,
            contract::{Bindings, ChangePassword, Login, Logout, Me, MfaConfirm, MfaEnrol},
            login, logout, me, mfa_confirm, mfa_enrol,
        },
        policy::SessionCookiePolicy,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings {
        login: Login::bind(login),
        logout: Logout::bind_authorized(SessionCookiePolicy, logout),
        me: Me::bind_authorized(SessionCookiePolicy, me),
        mfa_enrol: MfaEnrol::bind_authorized(SessionCookiePolicy, mfa_enrol),
        mfa_confirm: MfaConfirm::bind_authorized(SessionCookiePolicy, mfa_confirm),
        change_password: ChangePassword::bind_authorized(SessionCookiePolicy, change_password),
    }
    .mount(router)
}
