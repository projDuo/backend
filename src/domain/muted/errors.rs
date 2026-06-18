use crate::domain::InternalError;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum MutedError {
    #[error("User is already muted.")]
    AlreadyMuted,
    #[error("User is not muted.")]
    NotMuted,
    #[error("You cannot mute yourself.")]
    CannotMuteSelf,
    #[error("Internal server error.")]
    Internal(#[from] InternalError),
}
