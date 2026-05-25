use poem::http::StatusCode;

use super::error_code::*;
use crate::domain::game::errors::GameError;

impl RestError for GameError {
    fn rest_status(&self) -> poem::http::StatusCode {
        match self {
            Self::GameNotFound => StatusCode::NOT_FOUND,
            Self::NotEnoughPlayers => StatusCode::PRECONDITION_FAILED,
            Self::CardNotFound => StatusCode::NOT_FOUND,
            Self::PlayerNotFound => StatusCode::NOT_FOUND,
            Self::WrongTurn => StatusCode::FORBIDDEN,
            Self::WrongCard => StatusCode::BAD_REQUEST,
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (error_code, message) = match self {
            Self::GameNotFound => (
                "GAME_NOT_FOUND",
                "Couldn't find the requested game.",
            ),
            Self::NotEnoughPlayers => (
                "NOT_ENOUGH_PLAYERS",
                "Insufficient amount of players to start the game session."
            ),
            Self::CardNotFound => (
                "CARD_NOT_FOUND",
                "The requested card is not found.",
            ),
            Self::PlayerNotFound => (
                "PLAYER_NOT_FOUND",
                "The requested player is not found."
            ),
            Self::WrongTurn => (
                "WRONG_TURN",
                "This turn doesn't belong to the player."
            ),
            Self::WrongCard => (
                "WRONG_CARD",
                "The selected card can't be played per the rules."
            )
        };

        ErrorCode::new( error_code, message )
    }
}
impl_api_error!(GameError);