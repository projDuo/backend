//pub mod game;
pub mod accounts;
pub mod savefiles;
pub mod sessions;
pub mod auth;

use thiserror::Error;

pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;
pub trait Identifiable {
    type Id;
    fn id(&self) -> Self::Id;
}

pub trait ErrorType {
    type Error;
}



#[derive(Clone, Debug, Error)]
#[error("Internal repository error: {0}")]
pub struct InternalError(pub String);