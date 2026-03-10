use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("A record with this {0} already exists.")]
    UniqueViolation(String),

    #[error("Invalid reference provided for {0}.")]
    ForeignKeyViolation(String),

    #[error("The field '{0}' cannot be empty.")]
    NotNullViolation(String),

    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Internal database error: {0}")]
    Internal(String),
}