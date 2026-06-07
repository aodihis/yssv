use std::fmt;

#[derive(Debug, Clone)]
pub struct DbError {
    pub message: String,
    pub kind: DbErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbErrorKind {
    ConnectionRefused,
    AuthFailed,
    DatabaseNotFound,
    PermissionDenied,
    Other,
}

impl DbError {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let kind = classify(&message);
        tracing::warn!(error = %message, kind = ?kind, "database error");
        Self { message, kind }
    }

    /// Human-readable hint shown in the UI when connection fails.
    pub fn install_hint(&self) -> Option<&'static str> {
        match self.kind {
            DbErrorKind::ConnectionRefused => Some(
                "Could not reach the database server.\n\
                 Make sure the server is installed and running on the specified host and port.\n\
                 For PostgreSQL: check that `postgres` service is active.\n\
                 For MySQL/MariaDB: check that `mysqld` / `mariadbd` service is active.",
            ),
            DbErrorKind::AuthFailed => {
                Some("Authentication failed. Check your username and password.")
            }
            DbErrorKind::DatabaseNotFound => {
                Some("The specified database was not found. Check the database name.")
            }
            DbErrorKind::PermissionDenied => {
                Some("Permission denied. The user may not have access to this database.")
            }
            DbErrorKind::Other => None,
        }
    }
}

fn classify(msg: &str) -> DbErrorKind {
    let lower = msg.to_lowercase();
    if lower.contains("connection refused")
        || lower.contains("no route to host")
        || lower.contains("timed out")
        || lower.contains("network unreachable")
    {
        DbErrorKind::ConnectionRefused
    } else if lower.contains("password authentication failed")
        || lower.contains("access denied for user")
        || lower.contains("invalid password")
    {
        DbErrorKind::AuthFailed
    } else if lower.contains("database") && lower.contains("does not exist") {
        DbErrorKind::DatabaseNotFound
    } else if lower.contains("permission denied") {
        DbErrorKind::PermissionDenied
    } else {
        DbErrorKind::Other
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DbError {}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        DbError::new(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_connection_refused() {
        let e = DbError::new("Connection refused (os error 111)");
        assert_eq!(e.kind, DbErrorKind::ConnectionRefused);
        assert!(e.install_hint().is_some());
    }

    #[test]
    fn classifies_auth_failed() {
        let e = DbError::new("password authentication failed for user \"postgres\"");
        assert_eq!(e.kind, DbErrorKind::AuthFailed);
        assert!(e.install_hint().is_some());
    }

    #[test]
    fn classifies_database_not_found() {
        let e = DbError::new("database \"mydb\" does not exist");
        assert_eq!(e.kind, DbErrorKind::DatabaseNotFound);
    }

    #[test]
    fn classifies_other() {
        let e = DbError::new("some unexpected internal error");
        assert_eq!(e.kind, DbErrorKind::Other);
        assert!(e.install_hint().is_none());
    }
}
