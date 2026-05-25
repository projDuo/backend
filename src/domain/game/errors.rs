#[derive(Debug, thiserror::Error)]
pub enum GameError { //Список усіх можливих помилок
    #[error("Couldn't find the requested game.")]
    GameNotFound,
    #[error("Insufficient amount of players to start the game session.")]
    NotEnoughPlayers,
    #[error("The requested card is not found.")]
    CardNotFound,
    #[error("The requested player is not found.")]
    PlayerNotFound,
    #[error("This turn doesn't belong to the player.")]
    WrongTurn,
    #[error("The selected card can't be played per the rules.")]
    WrongCard,
}