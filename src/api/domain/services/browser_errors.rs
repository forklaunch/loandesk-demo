//! Errors reported by the front end (uncaught exceptions, rejected promises,
//! rendering failures). Each accepted report becomes one structured
//! `browser_error` log line, which travels the same OTLP pipeline as the
//! services' own errors and has an alert rule
//! (`src/monitoring/alerts/loandesk.rules.yaml`).
//!
//! A report is input from a browser, so nothing in it is trusted: every field
//! is reduced to a fixed vocabulary or a narrow character set before it is
//! logged, and at most `MAX_PER_MINUTE` reports are logged per instance.
use crate::{
    api::{policy::CurrentUser, transport},
    domain::schemas::browser_errors::BrowserErrorReport,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const KINDS: [&str; 3] = ["error", "unhandledrejection", "render"];
const MAX_FRAMES: usize = 20;
const MAX_FRAME_CHARS: usize = 200;
const MAX_PER_MINUTE: u32 = 60;

/// A report reduced to what may be logged.
#[derive(Debug, PartialEq, Eq)]
pub struct Sanitized {
    pub kind: &'static str,
    pub class: String,
    pub screen: String,
    pub release: String,
    pub frames: Vec<String>,
}

fn identifier(value: &str, fallback: &str) -> String {
    let ok = !value.is_empty()
        && value.len() <= 80
        && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '.' | '-'));
    if ok { value.to_owned() } else { fallback.to_owned() }
}

/// Route with anything that is not a fixed word replaced, so a record id (or
/// anything else that found its way into the address bar) is not logged.
fn screen(value: &str) -> String {
    let path = value.split(['?', '&']).next().unwrap_or("");
    let parts: Vec<String> = path
        .trim_start_matches('#')
        .split('/')
        .filter(|part| !part.is_empty())
        .take(6)
        .map(|part| {
            let word = part.len() <= 24 && part.chars().all(|c| c.is_ascii_lowercase() || c == '-');
            if word { part.to_owned() } else { ":id".to_owned() }
        })
        .collect();
    format!("#/{}", parts.join("/"))
}

/// Keeps lines that look like stack frames ("at fn (file.js:1:2)" or
/// "fn@file.js:1:2") and are made only of characters such frames use. The
/// first line of a browser stack repeats the error message; it is not a frame
/// and is dropped with everything else that does not fit.
fn frames(stack: &str) -> Vec<String> {
    stack
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("at ") || line.contains('@'))
        .filter(|line| line.len() <= MAX_FRAME_CHARS)
        .filter(|line| {
            line.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || matches!(
                        c,
                        ' ' | '_'
                            | '$'
                            | '.'
                            | '<'
                            | '>'
                            | '/'
                            | ':'
                            | '@'
                            | '('
                            | ')'
                            | '-'
                            | '['
                            | ']'
                    )
            })
        })
        .filter(|line| {
            line.rsplit(':')
                .next()
                .is_some_and(|tail| tail.trim_end_matches(')').chars().all(|c| c.is_ascii_digit()))
        })
        .take(MAX_FRAMES)
        .map(str::to_owned)
        .collect()
}

pub fn sanitize(report: &BrowserErrorReport) -> Option<Sanitized> {
    let kind = KINDS.iter().find(|kind| **kind == report.kind).copied()?;
    Some(Sanitized {
        kind,
        class: identifier(&report.class, "Error"),
        screen: screen(&report.screen),
        release: identifier(&report.release, "unknown"),
        frames: frames(&report.stack),
    })
}

/// At most `limit` events per minute; the rest are counted, not logged.
struct Throttle {
    limit: u32,
    window: Mutex<(Instant, u32)>,
}
impl Throttle {
    fn admit(&self, now: Instant) -> bool {
        let mut window = self.window.lock().expect("throttle lock");
        if now.duration_since(window.0) >= Duration::from_secs(60) {
            *window = (now, 0);
        }
        window.1 += 1;
        window.1 <= self.limit
    }
}

#[derive(Clone)]
pub struct BrowserErrors {
    throttle: Arc<Throttle>,
}

impl Default for BrowserErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserErrors {
    pub fn new() -> Self {
        Self {
            throttle: Arc::new(Throttle {
                limit: MAX_PER_MINUTE,
                window: Mutex::new((Instant::now(), 0)),
            }),
        }
    }

    /// Returns whether the report was recorded. Malformed and excess reports
    /// are dropped; the caller is told nothing either way.
    pub fn record(&self, user: &CurrentUser, report: &BrowserErrorReport) -> bool {
        let Some(sanitized) = sanitize(report) else {
            return false;
        };
        if !self.throttle.admit(Instant::now()) {
            return false;
        }
        tracing::warn!(
            event = "browser_error",
            kind = sanitized.kind,
            class = %sanitized.class,
            screen = %sanitized.screen,
            release = %sanitized.release,
            frames = sanitized.frames.len(),
            stack = %sanitized.frames.join(" | "),
            user_id = %user.id,
            role = %user.role,
            request_id = %transport::request_id(),
            "the front end reported an error"
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::test_logs;
    use uuid::Uuid;

    fn report() -> BrowserErrorReport {
        BrowserErrorReport {
            kind: "unhandledrejection".into(),
            class: "TypeError".into(),
            screen: "#/applications/7f3c9a52-6f0e-4d53-9f38-0a4f3b0f8a11".into(),
            stack: "TypeError: SSN for Eleanor Whitfield must be 9 digits (900-55-1234)\n    \
                    at submit (http://127.0.0.1:41080/assets/index-ZXnfeADE.js:40:1234)\n    \
                    at Eleanor Whitfield <eleanor@example.com>\n    \
                    at http://127.0.0.1:41080/assets/index-ZXnfeADE.js:12:99?ssn=900551234\n\
                    render@http://127.0.0.1:41080/assets/index-ZXnfeADE.js:7:21"
                .into(),
            release: "0.1.0".into(),
        }
    }

    fn user() -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            email: "maya.chen@loandesk.demo".into(),
            full_name: "Maya Chen".into(),
            role: "relationship_manager".into(),
            mfa_enabled: false,
            session_hash: "hash".into(),
        }
    }

    #[test]
    fn only_stack_frames_survive_and_ids_leave_the_screen() {
        let sanitized = sanitize(&report()).expect("a valid report");
        assert_eq!(sanitized.kind, "unhandledrejection");
        assert_eq!(sanitized.screen, "#/applications/:id");
        assert_eq!(
            sanitized.frames,
            [
                "at submit (http://127.0.0.1:41080/assets/index-ZXnfeADE.js:40:1234)",
                "render@http://127.0.0.1:41080/assets/index-ZXnfeADE.js:7:21",
            ]
        );
        let unknown = BrowserErrorReport { kind: "anything".into(), ..report() };
        assert_eq!(sanitize(&unknown), None);
        let odd = BrowserErrorReport { class: "Eleanor Whitfield".into(), ..report() };
        assert_eq!(sanitize(&odd).unwrap().class, "Error");
    }

    #[test]
    fn a_report_is_logged_without_the_message_or_anything_a_user_typed() {
        let reporter = user();
        let logs = test_logs::capture(|| {
            assert!(BrowserErrors::new().record(&reporter, &report()));
        });
        assert!(logs.contains("browser_error") && logs.contains("TypeError"), "{logs}");
        assert!(logs.contains(&reporter.id.to_string()), "{logs}");
        for value in [
            "Eleanor",
            "Whitfield",
            "900-55-1234",
            "900551234",
            "eleanor@",
            "Maya",
            "maya.chen",
            "7f3c9a52",
        ] {
            assert!(!logs.contains(value), "`{value}` was written to the log: {logs}");
        }
    }

    #[test]
    fn reports_beyond_the_limit_are_dropped() {
        let throttle = Throttle { limit: 2, window: Mutex::new((Instant::now(), 0)) };
        let now = Instant::now();
        assert!(throttle.admit(now));
        assert!(throttle.admit(now));
        assert!(!throttle.admit(now));
        assert!(throttle.admit(now + Duration::from_secs(61)));
    }
}
