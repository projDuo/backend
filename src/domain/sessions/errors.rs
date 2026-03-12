
use thiserror::Error;

use super::InternalError;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("This token is already assigned.")]
    Duplicate,
    #[error("This token has expired.")]
    Expired,
    #[error("This token has been revoked")]
    Revoked,
    #[error("This token exists but is not yet valid.")]
    NotYetValid,
    #[error("Invalid token provided.")]
    Invalid,
    #[error("Database is unavailable")]
    Infrastructure(#[from] InternalError),
}