use thiserror::Error;

use crate::domain::accounts::errors::*;
use crate::domain::sessions::errors::*;
use crate::domain::InternalError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Account failure: {0}")]
    Account(#[from] AccountError),
    #[error("Session failure: {0}")]
    Session(#[from] SessionError),
    #[error("Database is unavailable")]
    Infrastructure(#[from] InternalError),
}