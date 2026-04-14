use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
}

impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => DomainError::NotFound("Record not found".into()),
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                DomainError::AlreadyExists("Duplicate record".into())
            }
            _ => DomainError::Infrastructure(e.to_string()),
        }
    }
}
