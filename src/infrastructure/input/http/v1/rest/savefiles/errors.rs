use poem::http::StatusCode;
use crate::domain::savefiles::errors::*;
use super::error_code::*;

impl RestError for SavefileError {
    fn rest_status(&self) -> StatusCode {
        match self {
            Self::Duplicate => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (code, msg) = match self {
            Self::NotFound => (
                "NOT_FOUND",
                "Savefile not found"
            ),
            Self::Duplicate => (
                "ALREADY_EXISTS",
                "A savefile with this id already exists."
            ),
            Self::Infrastructure(v) => { return v.rest_error() }
        };

        ErrorCode::new(code, msg)
    }
}
impl_api_error!(SavefileError);