//! Where the front end reports errors that happened in a user's browser.
use crate::{
    api::policy::CurrentUser,
    domain::schemas::{auth::Ok as OkBody, browser_errors::BrowserErrorReport},
    registrations::AppState,
};
use forklaunch_axum::native::{EndpointError, State};

pub async fn report(
    State(state): State<AppState>,
    user: CurrentUser,
    input: BrowserErrorReport,
) -> Result<OkBody, EndpointError> {
    // Always the same answer: a browser learns nothing about what was kept.
    state.browser_errors.record(&user, &input);
    Ok(OkBody { ok: true })
}

#[forklaunch_core::contract::api]
pub mod contract {
    use crate::domain::schemas::{auth::Ok, browser_errors::BrowserErrorReport};
    use forklaunch_core::contract::{
        HttpMethod,
        endpoint::{Endpoint, JsonBody},
    };

    #[access(Authenticated)]
    pub fn report() -> Endpoint<JsonBody<BrowserErrorReport>, JsonBody<Ok>> {
        Endpoint::new(
            "Report Browser Error",
            "Record an error the front end caught in the browser (class and stack frames only)",
            HttpMethod::Post,
            "/api/client-errors",
            200,
        )
        .expect("valid browser error endpoint")
    }
}
