use poem::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorCode {
    pub error_code: String,
    pub message: String,
}

impl ErrorCode {
    pub fn new(
        error_code: &str,
        message: &str,
    ) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
        }
    }
}

pub trait RestError {
    fn rest_status(&self) -> StatusCode { StatusCode::INTERNAL_SERVER_ERROR }
    fn rest_error(&self) -> ErrorCode { 
        ErrorCode::new(
            "INTERNAL_SERVER_ERROR",
    "Internal server error. Please try again later.",
        ) 
    }
}

impl RestError for InternalError {}
impl_api_error!(InternalError);

macro_rules! impl_api_error {
    ($error_type:ty) => {
        impl poem::error::ResponseError for $error_type {
            fn status(&self) -> poem::http::StatusCode {
                self.rest_status()
            }
            fn as_response(&self) -> poem::Response {
                poem::Response::builder()
                    .status(self.status())
                    .content_type("application/json")
                    .body(poem::Body::from_json(&self.rest_error()).unwrap())
            }
        }
    };
}
pub(super) use impl_api_error;

use crate::domain::InternalError;