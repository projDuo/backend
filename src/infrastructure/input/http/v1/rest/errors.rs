use poem::http::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
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

impl<E: RestError> From<E> for ErrorCode {
    fn from(error: E) -> Self {
        error.rest_error()
    }
}

impl RestError for InternalError {
    fn rest_error(&self) -> ErrorCode {
        eprintln!("{}", self);

        ErrorCode::new(
            "INTERNAL_SERVER_ERROR",
    "Internal server error. Please try again later.",
        ) 
    }
}
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

#[macro_export]
macro_rules! vec_try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(errs) => {
                use poem::IntoResponse;
                
                let error_codes: Vec<_> = errs
                    .iter()
                    .map(|v| v.rest_error())
                    .collect();
                
                let res = (poem::http::StatusCode::BAD_REQUEST, poem::web::Json(error_codes)).into_response();
                return Err(poem::Error::from_response(res));
            }
        }
    };
}
pub(super) use vec_try;