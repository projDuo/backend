//pub mod game;
pub mod accounts;
pub mod savefiles;
pub mod sessions;
pub mod auth;
pub mod game;

use thiserror::Error;

pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;


#[derive(Clone, Debug, Error)]
#[error("Internal repository error: {0}")]
pub struct InternalError(pub String);