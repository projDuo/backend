use poem::http::StatusCode;

use crate::{adapters::rest::v1::errors::{ErrorCode, RestError, impl_api_error}, domain::sessions::SessionError};



impl RestError for SessionError {
    fn rest_status(&self) -> poem::http::StatusCode {
        match self {
            Self::Duplicate => StatusCode::CONFLICT,
            Self::Expired | 
            Self::Revoked | 
            Self::NotYetValid |
            Self::Invalid => StatusCode::FORBIDDEN,
            Self::Infrastructure(v) => v.rest_status(),
        }
    }

    fn rest_error(&self) -> crate::adapters::rest::v1::errors::ErrorCode {
        let (code, msg) = match self {
            Self::Duplicate => (
                "ALREADY_EXISTS",
                "This token is already assigned.",
            ),
            Self::Expired => (
                "EXPIRED",
                "This token has expired."
            ),
            Self::Revoked |
            Self::NotYetValid |
            Self::Invalid => (
                "INVALID_TOKEN",
                "This token is either revoked, not yet valid or invalid."
            ),
            Self::Infrastructure(v) => { return v.rest_error(); }
        };

        ErrorCode::new(code, msg)
    }
}
impl_api_error!(SessionError);