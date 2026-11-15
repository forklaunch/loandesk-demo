//! A check on every log statement in the API, the worker and the shared crate.
//!
//! Logs leave the services (container logs, OTLP, Loki), so nothing a borrower
//! or a colleague entered may be written to them: services log ids, fixed
//! words, counts and booleans. This test reads the source of every `tracing`
//! call and fails when one could carry more than that:
//!
//! - no value is logged with `Debug` (`?value`): `Debug` of a driver error or
//!   a record prints whatever it holds;
//! - no field is computed from a request body, a record field that holds
//!   personal or confidential data, a credential, or an error that has not
//!   been through `logsafe::describe`;
//! - the message is a fixed sentence, with nothing interpolated into it.
//!
//! The behaviour of the two places that do log error text is tested where they
//! live (`error.rs`, `handler.rs`, `shared::logsafe`), and
//! `scripts/verify-operations.sh` searches the logs of a running stack for the
//! demo data's names, SSNs, dates of birth and passwords.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Identifiers that name data which must not reach a log line.
const FORBIDDEN: [&str; 38] = [
    "ssn",
    "ssn_last4",
    "full_name",
    "name",
    "owner_name",
    "owner_names",
    "legal_name",
    "email",
    "password",
    "hash",
    "dummy_hash",
    "purpose",
    "note",
    "decision_note",
    "filename",
    "label",
    "date_of_birth",
    "dob",
    "ein",
    "address",
    "industry",
    "body",
    "input",
    "payload",
    "report",
    "token",
    "tokens",
    "secret",
    "cookie",
    "plaintext",
    "ciphertext",
    "totp",
    "code",
    "text",
    "key",
    "q",
    "pattern",
    "headers",
];

/// Errors may be logged only through these.
const ERROR_FORMATTERS: [&str; 2] = ["logsafe::describe(", "describe("];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("{} is readable", dir.display())) {
        let path = entry.expect("directory entry").path();
        let name = path.file_name().expect("file name").to_string_lossy().into_owned();
        if path.is_dir() {
            // Generated SDKs and build output contain no hand-written log calls.
            if !["sdk", "target", "node_modules", "__test__", "tests"].contains(&name.as_str()) {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

/// The source text of every `tracing::<macro>!( ... )` call, without the test
/// modules, and whether it opens a span (which has a name and no message).
fn log_calls(source: &str) -> Vec<(bool, String)> {
    let source = source.split("#[cfg(test)]").next().unwrap_or("");
    let mut calls = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("tracing::") {
        rest = &rest[at + "tracing::".len()..];
        let Some(bang) = rest.find("!(") else { break };
        let name = &rest[..bang];
        if !["trace", "debug", "info", "warn", "error", "info_span", "warn_span", "error_span"]
            .contains(&name)
        {
            continue;
        }
        let body = &rest[bang + 2..];
        let (mut depth, mut in_string, mut escaped, mut end) = (1_usize, false, false, None);
        for (index, c) in body.char_indices() {
            if in_string {
                match c {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.expect("balanced log call");
        calls.push((name.ends_with("_span"), body[..end].to_owned()));
        rest = &body[end..];
    }
    calls
}

/// Splits a call body into its top-level, comma-separated arguments.
fn arguments(call: &str) -> Vec<String> {
    let (mut depth, mut in_string, mut escaped) = (0_i32, false, false);
    let mut parts = vec![String::new()];
    for c in call.chars() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else {
            match c {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(String::new());
                    continue;
                }
                _ => {}
            }
        }
        parts.last_mut().expect("one part").push(c);
    }
    parts.into_iter().map(|part| part.trim().to_owned()).filter(|part| !part.is_empty()).collect()
}

/// The code of a field value, with string literals blanked out.
fn code_only(value: &str) -> String {
    let (mut out, mut in_string, mut escaped) = (String::new(), false, false);
    for c in value.chars() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else if c == '"' {
            in_string = true;
        } else {
            out.push(c);
        }
    }
    out
}

fn violations(file: &str, span: bool, call: &str) -> Vec<String> {
    let mut found = Vec::new();
    let arguments = arguments(call);
    // An event ends with its message; a span starts with its name.
    let (message, fields) = if span {
        let Some((name, fields)) = arguments.split_first() else { return found };
        (name, fields)
    } else {
        let Some((message, fields)) = arguments.split_last() else { return found };
        (message, fields)
    };
    for field in fields {
        let Some((name, value)) = field.split_once('=') else {
            // A span name (a string literal) or a bare field (`operation`).
            let bare = field.trim_matches('"');
            if !field.starts_with('"') && FORBIDDEN.contains(&bare) {
                found.push(format!("{file}: logs `{bare}`"));
            }
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        if value.starts_with('?') {
            found.push(format!("{file}: `{name}` is logged with Debug (`{value}`)"));
        }
        let code = code_only(value);
        let through_describe = ERROR_FORMATTERS.iter().any(|formatter| code.contains(formatter));
        for word in code.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if FORBIDDEN.contains(&word) {
                found.push(format!("{file}: `{name}` is computed from `{word}` (`{value}`)"));
            }
            if word == "error" && !through_describe {
                found.push(format!(
                    "{file}: `{name}` logs an error without logsafe::describe (`{value}`)"
                ));
            }
        }
    }
    if !(message.starts_with('"') && message.ends_with('"')) {
        found.push(format!("{file}: the message is not a fixed sentence (`{message}`)"));
    } else if message.contains('{') {
        found.push(format!("{file}: the message interpolates a value (`{message}`)"));
    }
    found
}

fn sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("src directory");
    let mut files = Vec::new();
    for service in ["api", "extraction", "shared"] {
        rust_files(&root.join(service), &mut files);
    }
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let name = path.strip_prefix(root).expect("under src").to_string_lossy().into_owned();
            (name, fs::read_to_string(&path).expect("readable source"))
        })
        .collect()
}

#[test]
fn no_log_statement_can_carry_personal_data_a_credential_or_a_raw_error() {
    let mut calls = 0;
    let mut problems = Vec::new();
    for (file, source) in sources() {
        for (span, call) in log_calls(&source) {
            calls += 1;
            problems.extend(violations(&file, span, &call));
        }
    }
    // A floor well under the real number, so a scan that silently stops finding
    // calls fails instead of passing with nothing checked.
    assert!(calls >= 30, "the scan found only {calls} log statements; is it still looking?");
    assert!(problems.is_empty(), "unsafe log statements:\n{}", problems.join("\n"));
}

#[test]
fn nothing_is_printed_outside_the_structured_logger() {
    // `println!` is for the command-line tools' own answers (job ids, counts).
    // `dbg!` and `eprintln!` would bypass the logger and its rules.
    for (file, source) in sources() {
        let code = source.split("#[cfg(test)]").next().unwrap_or("");
        for banned in ["dbg!(", "eprintln!(", "eprint!("] {
            assert!(!code.contains(banned), "{file} uses {banned}");
        }
    }
}

#[test]
fn the_scan_catches_what_it_is_meant_to_catch() {
    let unsafe_calls = [
        r#"event = "x", error_detail = ?error, "failed""#,
        r#"event = "x", owner = %owner.full_name, "created""#,
        r#"event = "x", ssn = %ssn, "revealed""#,
        r#"event = "x", who = %input.email, "sign-in""#,
        r#"event = "x", error = %error, "failed""#,
        r#"event = "x", "created {name}""#,
        r#"event = "x", body = ?body, "request""#,
    ];
    for call in unsafe_calls {
        assert!(!violations("example.rs", false, call).is_empty(), "not caught: {call}");
    }
    let safe_calls = [
        r#"event = "permission_denied", user_id = %user.id, role = %user.role, action, resource = %id, "permission denied""#,
        r#"event = "dependency_error", operation, error = %shared::logsafe::describe(&error), "dependency call failed""#,
        r#"event = "job_completed", document = %document, revenue = figures.revenue.is_some(), "document processed""#,
    ];
    for call in safe_calls {
        assert_eq!(violations("example.rs", false, call), Vec::<String>::new(), "{call}");
    }
    let span = r#""extraction_job", job_id = %job.id, attempt"#;
    assert_eq!(violations("example.rs", true, span), Vec::<String>::new());
    assert!(!violations("example.rs", true, r#""job", file = %filename"#).is_empty());
}
