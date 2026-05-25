use poem::http::StatusCode;

use crate::domain::activity::errors::*;
use super::error_code::*;

impl RestError for ActivityError {
    fn rest_status(&self) -> poem::http::StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Infrastructure(v) => v.rest_status(),
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (error_code, message) = match self {
            Self::NotFound => (
                "GATEWAY_UNAUTHORIZED",
                "You must be authorized through gateway to use this endpoint."
            ),
            Self::Infrastructure(v) => return v.rest_error(),
        };

        ErrorCode::new(error_code, message)
    }
}
impl_api_error!(ActivityError);