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

/// Logs a failed dependency call with its operation and full error chain,
/// then hides the detail from the caller.
pub fn storage<E: fmt::Debug + fmt::Display>(
    operation: &'static str,
) -> impl FnOnce(E) -> AppError {
    move |error| {
        tracing::error!(
            event = "dependency_error",
            operation,
            request_id = %transport::request_id(),
            error = %error,
            error_detail = ?error,
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
