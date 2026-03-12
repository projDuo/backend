use thiserror::Error;
use super::InternalError;

#[derive(Clone, Debug, Error)]
#[error("A password must be at least 8 characters long and consist of letters, special characters and numbers.")]
pub struct WeakPassword;

#[derive(Clone, Debug, Error)]
#[error("TBD")]
pub struct InvalidLogin;

#[derive(Clone, Debug, Error)]
#[error("Incorrect display name provided.")]
pub struct InvalidDisplayName;

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Account not found")]
    NotFound,
    #[error("Such an account already exists.")]
    Duplicate,
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    #[error("Database is unavailable")]
    Infrastructure(#[from] InternalError),
}