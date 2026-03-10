use thiserror::Error;
use uuid::Uuid;
use crate::domain::{DateTimeWithTimeZone, ErrorType};
use super::Identifiable;

pub struct Account {
    pub id: Uuid,
    pub login: String,
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl Identifiable for Account {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    #[error("Incorrect display name provided.")]
    InvalidDisplayName,
    #[error("A password must be at least 6 characters long")]
    WeakPassword,
}

impl ErrorType for Account {
    type Error = Error;
}