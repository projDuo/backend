use poem::http::StatusCode;
use crate::domain::accounts::errors::*;
use super::error_code::*;

impl RestError for WeakPassword {
    fn rest_status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn rest_error(&self) -> ErrorCode {
        ErrorCode::new(
            "WEAK_PASSWORD",
            "A password must be at least 8 characters long and consist of letters, special characters and numbers."
        )
    }
}
impl_api_error!(WeakPassword);

impl RestError for InvalidLogin {
    fn rest_status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn rest_error(&self) -> ErrorCode {
        ErrorCode::new(
            "INVALID_LOGIN",
            "TBD"
        )
    }
}
impl_api_error!(InvalidLogin);

impl RestError for InvalidDisplayName {
    fn rest_status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn rest_error(&self) -> ErrorCode {
        ErrorCode::new(
            "INVALID_DISPLAY_NAME",
            "Incorrect display name provided."
        )
    }
}
impl_api_error!(InvalidDisplayName);

impl RestError for AccountError {
    fn rest_status(&self) -> StatusCode {
        match self {
            AccountError::NotFound => StatusCode::NOT_FOUND,
            AccountError::Duplicate => StatusCode::CONFLICT,
            AccountError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AccountError::Infrastructure(v) => v.rest_status(),
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (code, msg) = match self {
            AccountError::NotFound => (
                "NOT_FOUND",
                "Account not found."
            ),
            AccountError::Duplicate => (
                "DUPLICATE",
                "Such an account already exists."
            ),
            AccountError::InvalidCredentials => (
                "INVALID_CREDENTIALS",
                "Invalid credentials provided."
            ),
            AccountError::Infrastructure(v) => {
                return v.rest_error();
            },
        };
        
        ErrorCode::new(
            code,
            msg
        )
    }
}
impl_api_error!(AccountError);