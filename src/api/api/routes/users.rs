use crate::{
    api::{
        controllers::users::{
            contract::{Bindings, CreateUser, ListUsers, SetRole},
            create_user, list_users, set_role,
        },
        policy::SessionCookiePolicy,
    },
    registrations::AppState,
};
use forklaunch_axum::native::ContractRouter;
pub fn mount(router: ContractRouter<AppState>) -> Result<ContractRouter<AppState>, String> {
    Bindings {
        list_users: ListUsers::bind_authorized(SessionCookiePolicy, list_users),
        create_user: CreateUser::bind_authorized(SessionCookiePolicy, create_user),
        set_role: SetRole::bind_authorized(SessionCookiePolicy, set_role),
    }
    .mount(router)
}
