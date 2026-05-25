use poem::http::StatusCode;
use crate::domain::room::errors::*;
use super::error_code::*;

impl RestError for FieldError {
    fn rest_status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn rest_error(&self) -> ErrorCode {
        let (code, message) =  match self {
            FieldError::InvalidNameFormat => (
                "INVALID_NAME_FORMAT", 
                "Name cannot be empty or longer than 32 characters."
            ),
            FieldError::InvalidPasswordFormat => (
                "INVALID_PASSWORD_FORMAT", 
                "Password cannot be empty or longer than 32 characters."
            ),
            FieldError::OutOfRangeMaxPlayers => (
                "OUT_OF_RANGE_MAX_PLAYERS", 
                "Room can have 4 players at most."
            ),
        };

        ErrorCode::new(
            code,
            message
        )
    }
}
impl_api_error!(FieldError);

impl RestError for RoomError {
    fn rest_status(&self) -> StatusCode {
        match self {
            RoomError::RoomNotFound => StatusCode::NOT_FOUND,
            RoomError::WrongPassword => StatusCode::UNAUTHORIZED,
            RoomError::MaxPlayersReached => StatusCode::FORBIDDEN,
            RoomError::PlayerNotFound => StatusCode::NOT_FOUND,
            RoomError::PlayerNotInRoom => StatusCode::NOT_FOUND,
            RoomError::PlayerInRoom => StatusCode::FORBIDDEN,
            RoomError::PlayerInGame => StatusCode::FORBIDDEN,
            RoomError::NotAnOwner => StatusCode::FORBIDDEN,
            RoomError::FieldError(e) => e.rest_status(),
            RoomError::Infrastructure(e) => e.rest_status(),
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (code, message) = match self {
            RoomError::RoomNotFound => (
                "ROOM_NOT_FOUND",
                "Room with given id doesn't exist."
            ),
            RoomError::WrongPassword => (
                "WRONG_PASSWORD",
                "Wrong password to the room provided."
            ),
            RoomError::MaxPlayersReached => (
                "MAX_PLAYERS_REACHED",
                "The room already has max amount of players."
            ),
            RoomError::PlayerNotFound => (
                "PLAYER_NOT_FOUND",
                "This player is not in the room."
            ),
            RoomError::PlayerNotInRoom => (
                "PLAYER_NOT_IN_ROOM",
                "This player is not in any room."
            ),
            RoomError::PlayerInRoom => (
                "PLAYER_IN_ROOM",
                "This player is already in a room."
            ),
            RoomError::PlayerInGame => (
                "PLAYER_IN_GAME",
                "Cannot change ready state while in a game."
            ),
            RoomError::NotAnOwner => (
                "NOT_AN_OWNER",
                "This player is not an owner to perform the action."
            ),
            RoomError::FieldError(e) => return e.rest_error(),
            RoomError::Infrastructure(e) => return e.rest_error(),
        };

        ErrorCode::new(
            code,
            message
        )
    }
}
impl_api_error!(RoomError);