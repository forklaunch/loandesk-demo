//! Service errors. Controllers turn them into the framework's fixed endpoint
//! errors; the human-readable message the front end shows travels separately
//! (see `api::transport`) and never contains internal details.
use crate::api::transport;
use forklaunch_axum::native::EndpointError;
use forklaunch_core::{common::FrameworkError, config::ConfigError};
use std::{error::Error, fmt, result};

pub type Result<T> = result::Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidConfig(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound,
    BadRequest(String),
    Conflict(String),
    TooManyRequests(String),
    /// A dependency failed. `operation` says what was being done; the cause
    /// is logged where it happened and is never sent to the client.
    Storage(String),
}

pub fn bad(message: impl Into<String>) -> AppError {
    AppError::BadRequest(message.into())
}

/// Logs a failed dependency call with its operation and a description of the
/// error that is safe to log (`shared::logsafe`: never `Debug`, never a
/// database message or detail, which can quote row contents), then hides the
/// detail from the caller.
pub fn storage<E: fmt::Display + 'static>(operation: &'static str) -> impl FnOnce(E) -> AppError {
    move |error| {
        tracing::error!(
            event = "dependency_error",
            operation,
            request_id = %transport::request_id(),
            error = %shared::logsafe::describe(&error),
            "dependency call failed"
        );
        AppError::Storage(operation.to_owned())
    }
}

impl AppError {
    /// Maps to the framework error and records the message for the response.
    pub fn into_endpoint(self) -> EndpointError {
        let (error, message) = match self {
            Self::Unauthorized(message) => (EndpointError::Unauthorized, message),
            Self::Forbidden(message) => (EndpointError::Forbidden, message),
            Self::NotFound => (EndpointError::NotFound, "Not found".to_owned()),
            Self::BadRequest(message) => (EndpointError::InvalidRequest, message),
            Self::Conflict(message) => (EndpointError::Conflict, message),
            Self::TooManyRequests(message) => (EndpointError::TooManyRequests, message),
            Self::InvalidConfig(_) | Self::Storage(_) => {
                (EndpointError::Unavailable, "Something went wrong".to_owned())
            }
        };
        transport::set_error_message(message);
        error
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(formatter, "invalid config: {message}"),
            Self::Unauthorized(message) => write!(formatter, "unauthorized: {message}"),
            Self::Forbidden(message) => write!(formatter, "forbidden: {message}"),
            Self::NotFound => write!(formatter, "not found"),
            Self::BadRequest(message) => write!(formatter, "bad request: {message}"),
            Self::Conflict(message) => write!(formatter, "conflict: {message}"),
            Self::TooManyRequests(message) => write!(formatter, "too many requests: {message}"),
            Self::Storage(operation) => write!(formatter, "storage error during {operation}"),
        }
    }
}

impl Error for AppError {}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::InvalidConfig(error.to_string())
    }
}

impl From<FrameworkError> for AppError {
    fn from(error: FrameworkError) -> Self {
        storage("framework")(error)
    }
}

#[cfg(test)]
pub(crate) mod test_logs {
    //! Captures what a piece of code writes to the log, as text.
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    #[derive(Clone, Default)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);
    impl io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("log buffer").extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    pub fn capture(run: impl FnOnce()) -> String {
        let buffer = Buffer::default();
        let writer = buffer.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, run);
        String::from_utf8(buffer.0.lock().expect("log buffer").clone()).expect("utf-8 log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::logsafe::testing::{ROW_VALUES, row_violation};

    #[test]
    fn a_failed_database_call_is_logged_without_row_contents() {
        // The driver error quotes an owner's name, SSN and date of birth.
        let logs = test_logs::capture(|| {
            assert_eq!(
                storage("create owner")(row_violation()),
                AppError::Storage("create owner".into())
            );
        });
        assert!(logs.contains("dependency_error"), "{logs}");
        assert!(logs.contains("create owner") && logs.contains("sqlstate=23514"), "{logs}");
        assert!(logs.contains("constraint=owners_ownership_percent_check"), "{logs}");
        for value in ROW_VALUES {
            assert!(!logs.contains(value), "`{value}` was written to the log: {logs}");
        }
    }

    #[test]
    fn dependency_failures_reach_the_client_as_a_generic_message() {
        assert_eq!(
            AppError::Storage("create owner".into()).to_string(),
            "storage error during create owner"
        );
    }
}
