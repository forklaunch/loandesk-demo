//! What may be written to a log about a failed dependency call.
//!
//! Driver errors can carry data. A PostgreSQL constraint violation, for one,
//! reports the offending row ("Failing row contains (...)") or key
//! ("Key (email)=(...) already exists") in its detail, and some messages quote
//! the rejected value. Logging such an error with `{:?}` would put a
//! borrower's or a colleague's details into the log stream. Both services
//! therefore log errors only through [`describe`], which never uses `Debug`
//! and never copies a database message or detail.
use std::{any::Any, fmt::Display};

/// Longest description that is logged for an error this module does not know.
const MAX_CHARS: usize = 200;

/// A bounded description of `error` that is safe to log.
///
/// - Database errors: the SQLSTATE code and the constraint and table names
///   only. The server's message and detail are dropped.
/// - Other SQL driver errors: the kind of failure (pool timeout, I/O, ...).
/// - Anything else: its `Display` text, truncated. `Debug` is never used.
pub fn describe<E: Display + 'static>(error: &E) -> String {
    match (error as &dyn Any).downcast_ref::<sqlx::Error>() {
        Some(error) => describe_sql(error),
        None => truncate(&error.to_string()),
    }
}

fn describe_sql(error: &sqlx::Error) -> String {
    use sqlx::Error;
    match error {
        Error::Database(database) => {
            let mut text =
                format!("database error sqlstate={}", database.code().as_deref().unwrap_or("-"));
            for (label, value) in
                [("constraint", database.constraint()), ("table", database.table())]
            {
                if let Some(value) = value {
                    text.push_str(&format!(" {label}={value}"));
                }
            }
            text
        }
        Error::Io(error) => format!("database connection error: {:?}", error.kind()),
        Error::PoolTimedOut => "database pool timed out".to_owned(),
        Error::PoolClosed => "database pool closed".to_owned(),
        Error::RowNotFound => "row not found".to_owned(),
        Error::WorkerCrashed => "database worker crashed".to_owned(),
        Error::Tls(_) => "database TLS error".to_owned(),
        Error::Protocol(_) => "database protocol error".to_owned(),
        Error::Configuration(_) => "database configuration error".to_owned(),
        Error::ColumnNotFound(_) | Error::ColumnIndexOutOfBounds { .. } => {
            "database column not found".to_owned()
        }
        Error::ColumnDecode { .. } | Error::Decode(_) | Error::Encode(_) => {
            "database value could not be converted".to_owned()
        }
        Error::Migrate(_) => "database migration error".to_owned(),
        _ => "database error".to_owned(),
    }
}

fn truncate(text: &str) -> String {
    let mut out: String = text.chars().take(MAX_CHARS).collect();
    if text.chars().count() > MAX_CHARS {
        out.push('…');
    }
    out
}

/// A driver error shaped like a PostgreSQL constraint violation whose message
/// and detail carry row contents, for tests that prove such values stay out of
/// logs. Not compiled into the services.
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use sqlx::error::{DatabaseError, ErrorKind};
    use std::{borrow::Cow, error::Error as StdError, fmt};

    /// Values the fake violation carries; none may appear in a log line.
    pub const ROW_VALUES: [&str; 5] =
        ["Eleanor", "Whitfield", "900-55-1234", "1971-03-09", "Failing row"];

    #[derive(Debug)]
    struct Violation {
        message: String,
        #[allow(dead_code)]
        detail: String,
    }
    impl fmt::Display for Violation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.message)
        }
    }
    impl StdError for Violation {}
    impl DatabaseError for Violation {
        fn message(&self) -> &str {
            &self.message
        }
        fn code(&self) -> Option<Cow<'_, str>> {
            Some(Cow::Borrowed("23514"))
        }
        fn constraint(&self) -> Option<&str> {
            Some("owners_ownership_percent_check")
        }
        fn table(&self) -> Option<&str> {
            Some("owners")
        }
        fn kind(&self) -> ErrorKind {
            ErrorKind::CheckViolation
        }
        fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
            self
        }
        fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
            self
        }
        fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
            self
        }
    }

    pub fn row_violation() -> sqlx::Error {
        sqlx::Error::Database(Box::new(Violation {
            message: "invalid input syntax for type date: \"Eleanor Whitfield\"".into(),
            detail: "Failing row contains (Eleanor Whitfield, 900-55-1234, 1971-03-09).".into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        testing::{ROW_VALUES, row_violation},
        *,
    };

    #[test]
    fn a_database_error_is_described_without_its_message_or_row_contents() {
        let error = row_violation();
        // The raw error does carry the values: this is what must not be logged.
        assert!(format!("{error:?}").contains("900-55-1234"));
        assert!(error.to_string().contains("Eleanor Whitfield"));

        let described = describe(&error);
        assert_eq!(
            described,
            "database error sqlstate=23514 constraint=owners_ownership_percent_check table=owners"
        );
        for value in ROW_VALUES {
            assert!(!described.contains(value), "`{value}` reached the log description");
        }
    }

    #[test]
    fn driver_errors_are_named_and_other_errors_are_bounded() {
        assert_eq!(describe(&sqlx::Error::PoolTimedOut), "database pool timed out");
        assert_eq!(describe(&sqlx::Error::Protocol("secret".into())), "database protocol error");
        let long = "x".repeat(1000);
        assert_eq!(describe(&long).chars().count(), MAX_CHARS + 1);
        assert_eq!(describe(&"storage unavailable"), "storage unavailable");
    }
}
