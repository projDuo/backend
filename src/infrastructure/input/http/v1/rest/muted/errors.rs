use poem::http::StatusCode;

use crate::domain::muted::MutedError;
use super::super::errors::{ErrorCode, RestError};
use super::super::errors::impl_api_error;

impl RestError for MutedError {
    fn rest_status(&self) -> StatusCode {
        match self {
            MutedError::AlreadyMuted => StatusCode::CONFLICT,
            MutedError::NotMuted => StatusCode::NOT_FOUND,
            MutedError::CannotMuteSelf => StatusCode::BAD_REQUEST,
            MutedError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (error_code, message) = match self {
            MutedError::AlreadyMuted => ("MUTED_ALREADY_EXISTS", "This user is already muted."),
            MutedError::NotMuted => ("MUTED_NOT_FOUND", "This user is not muted."),
            MutedError::CannotMuteSelf => ("MUTED_CANNOT_SELF", "You cannot mute yourself."),
            MutedError::Internal(_) => ("INTERNAL_SERVER_ERROR", "Internal server error. Please try again later."),
        };

        ErrorCode::new(error_code, message)
    }
}

impl_api_error!(MutedError);
