//! The outermost HTTP layer, wrapped around the ForkLaunch router.
//!
//! - gives every request an id (kept from `x-request-id` when well-formed),
//!   returns it, and puts it on every log line written while handling it;
//! - logs each request with method, route, status and duration;
//! - carries the two things a typed contract handler cannot set itself: the
//!   user-facing error message and the session cookie;
//! - adds HSTS and a restrictive Content-Security-Policy (the framework adds
//!   nosniff, frame denial, no-referrer and no-store);
//! - keeps the original CSRF backstop: state-changing requests must carry
//!   `X-Requested-With: loandesk`, which browsers do not send cross-site.
use crate::registrations::AppState;
use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Default)]
struct Slot {
    request_id: String,
    traceparent: Option<String>,
    error_message: Mutex<Option<String>>,
    set_cookie: Mutex<Option<String>>,
    user_id: Mutex<Option<Uuid>>,
}

tokio::task_local! {
    static SLOT: Arc<Slot>;
}

/// The current request's id, or "-" outside a request (startup, tests).
pub fn request_id() -> String {
    SLOT.try_with(|slot| slot.request_id.clone()).unwrap_or_else(|_| "-".to_owned())
}
/// The caller's W3C trace context, forwarded into queued jobs.
pub fn traceparent() -> Option<String> {
    SLOT.try_with(|slot| slot.traceparent.clone()).ok().flatten()
}
pub fn set_error_message(message: String) {
    let _ = SLOT.try_with(|slot| *slot.error_message.lock().expect("slot lock") = Some(message));
}
pub fn set_cookie(cookie: String) {
    let _ = SLOT.try_with(|slot| *slot.set_cookie.lock().expect("slot lock") = Some(cookie));
}
pub fn set_user(id: Uuid) {
    let _ = SLOT.try_with(|slot| *slot.user_id.lock().expect("slot lock") = Some(id));
}

fn accepted_request_id(request: &Request) -> String {
    request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            (8..=64).contains(&value.len())
                && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .map_or_else(|| Uuid::new_v4().to_string(), str::to_owned)
}

/// Path with ids replaced, so logs and metrics group by route, not by record.
fn route_template(path: &str) -> String {
    path.split('/')
        .map(|part| if Uuid::parse_str(part).is_ok() { ":id" } else { part })
        .collect::<Vec<_>>()
        .join("/")
}

pub async fn layer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let route = route_template(request.uri().path());
    let slot = Arc::new(Slot {
        request_id: accepted_request_id(&request),
        traceparent: request
            .headers()
            .get("traceparent")
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.len() <= 64)
            .map(str::to_owned),
        ..Slot::default()
    });
    let span = tracing::info_span!("http_request", request_id = %slot.request_id);

    let safe = matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);
    let has_header =
        request.headers().get("x-requested-with").is_some_and(|value| value == "loandesk");
    let mut response = if safe || has_header {
        SLOT.scope(slot.clone(), next.run(request)).instrument(span.clone()).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Missing X-Requested-With header" })),
        )
            .into_response()
    };

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        if let Some(message) = slot.error_message.lock().expect("slot lock").take() {
            let headers = response.headers().clone();
            response = (status, Json(serde_json::json!({ "error": message }))).into_response();
            for (name, value) in &headers {
                if name != header::CONTENT_LENGTH && name != header::CONTENT_TYPE {
                    response.headers_mut().insert(name, value.clone());
                }
            }
        }
    }
    let headers = response.headers_mut();
    if let Some(cookie) = slot.set_cookie.lock().expect("slot lock").take()
        && let Ok(value) = HeaderValue::from_str(&cookie)
    {
        headers.insert(header::SET_COOKIE, value);
    }
    if let Ok(value) = HeaderValue::from_str(&slot.request_id) {
        headers.insert("x-request-id", value);
    }
    // API responses are JSON only: nothing may be loaded, framed or submitted from them.
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
        ),
    );
    if state.settings.cookie_secure {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    let user_id = slot.user_id.lock().expect("slot lock").map(|id| id.to_string());
    let _entered = span.enter();
    tracing::info!(
        event = "http_request",
        method = %method,
        route = %route,
        status = status.as_u16(),
        duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        user_id = user_id.as_deref().unwrap_or("-"),
        "request completed"
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_logged_as_templates_not_record_ids() {
        assert_eq!(
            route_template("/api/applications/7f3c9a52-6f0e-4d53-9f38-0a4f3b0f8a11/decision"),
            "/api/applications/:id/decision"
        );
        assert_eq!(route_template("/api/users"), "/api/users");
    }
}
