use poem::http::StatusCode;
use crate::domain::accounts as domain;
use super::errors_generic::{self, DomainErrorMapping};

impl DomainErrorMapping for domain::Error {
    fn to_http(&self) -> (StatusCode, &'static str, &'static str) {
        match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS", "Invalid credentials provided."),
            Self::WeakPassword => (StatusCode::BAD_REQUEST, "WEAK_PASSWORD", "Password must consist of at least 6 characters."),
            Self::InvalidDisplayName => (StatusCode::BAD_REQUEST, "INVALID_DISPLAY_NAME", "Display name is invalid or contains restricted characters.")
        }
    }
}

pub type Error = errors_generic::Error<domain::Error>;