use serde::Serialize;
use thiserror::Error;
use std::fmt;
use poem::{Response, http::StatusCode, error::ResponseError};
use crate::service;

#[derive(Serialize)]
pub struct ErrorCode {
    pub error_code: String,
    pub message: String,
}

pub trait DomainErrorMapping {
    fn to_http(&self) -> (StatusCode, &'static str, &'static str);
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error<T>(#[from] pub service::Error<T>);

impl<T: DomainErrorMapping + fmt::Debug + Send + Sync + 'static> ResponseError for Error<T> {
    fn status(&self) -> StatusCode {
        self.get_mapping().0
    }

    fn as_response(&self) -> Response {
        let (status, code, message) = self.get_mapping();

        let payload = ErrorCode {
            error_code: code.to_string(),
            message: message.to_string(),
        };

        Response::builder()
            .status(status)
            .content_type("application/json")
            .body(serde_json::to_string(&payload).unwrap())
    }
}

impl<T: DomainErrorMapping> Error<T> {
    fn get_mapping(&self) -> (StatusCode, &'static str, &'static str) {
        match &self.0 {
            // Because this is generic for the whole app, we use generic messages here!
            service::Error::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "The requested resource could not be found."),
            service::Error::Exists => (StatusCode::CONFLICT, "ALREADY_EXISTS", "This resource already exists."),
            service::Error::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", "Internal server error. Please try again later."),
            service::Error::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Insufficient level of access."),
            service::Error::NotModified => (StatusCode::NOT_MODIFIED, "NOT_MODIFIED", "No changes were made."),
            service::Error::Domain(e) => e.to_http(),
        }
    }
}