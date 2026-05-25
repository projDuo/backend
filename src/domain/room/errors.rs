use super::InternalError;

#[derive(Debug, thiserror::Error, Copy, Clone)]
pub enum FieldError {
    #[error("Name cannot be empty or longer than 32 characters.")]
    InvalidNameFormat,
    #[error("Password cannot be empty or longer than 32 characters.")]
    InvalidPasswordFormat,
    #[error("Room can have 4 players at most.")]
    OutOfRangeMaxPlayers
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum RoomError {
    #[error("Room with given id doesn't exist.")]
    RoomNotFound,
    #[error("Wrong password to the room provided.")]
    WrongPassword,
    #[error("The room already has max amount of players.")]
    MaxPlayersReached,
    #[error("This player is not in the room.")]
    PlayerNotFound,
    #[error("This player is already in a room.")]
    PlayerInRoom,
    #[error("This player is not in any room.")]
    PlayerNotInRoom,
    #[error("This player is not an owner to perform the action.")]
    NotAnOwner,
    #[error("Cannot change ready state while in a game.")]
    PlayerInGame,
    #[error("Field error: {0}")]
    FieldError(#[from] FieldError),
    #[error("Internal repository error: {0}")]
    Infrastructure(#[from] InternalError),
}
