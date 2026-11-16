use crate::{
    api::{
        controllers::users::{
            access_review,
            contract::{
                AccessReview, Bindings, CreateUser, DisableUser, EnableUser, ListAuditLog,
                ListUsers, RecordAccessReview, SetRole,
            },
            create_user, disable_user, enable_user, list_audit_log, list_users,
            record_access_review, set_role,
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
        disable_user: DisableUser::bind_authorized(SessionCookiePolicy, disable_user),
        enable_user: EnableUser::bind_authorized(SessionCookiePolicy, enable_user),
        access_review: AccessReview::bind_authorized(SessionCookiePolicy, access_review),
        list_audit_log: ListAuditLog::bind_authorized(SessionCookiePolicy, list_audit_log),
        record_access_review: RecordAccessReview::bind_authorized(
            SessionCookiePolicy,
            record_access_review,
        ),
    }
    .mount(router)
}
