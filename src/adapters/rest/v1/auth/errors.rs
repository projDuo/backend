use crate::{adapters::rest::v1::errors::{RestError, impl_api_error}, domain::auth::AuthError};



impl RestError for AuthError {
    fn rest_status(&self) -> poem::http::StatusCode {
        match self {
            AuthError::Account(v) => v.rest_status(),
            AuthError::Session(v) => v.rest_status(),
            AuthError::Infrastructure(v) => v.rest_status(),
        }
    }

    fn rest_error(&self) -> crate::adapters::rest::v1::errors::ErrorCode {
        match self {
            AuthError::Account(v) => v.rest_error(),
            AuthError::Session(v) => v.rest_error(),
            AuthError::Infrastructure(v) => v.rest_error(),
        }
    }
}
impl_api_error!(AuthError);