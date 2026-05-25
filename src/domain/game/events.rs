use uuid::Uuid;

use super::*;

#[derive(Debug, Clone)]
pub enum GameInternalEvents {
    RequestNewGameSession{player_ids: Vec<Uuid>},
    RemovePlayer{ game_id: Uuid, player_id: Uuid },
}

#[derive(Debug, Clone)]
pub enum GameEvents {
    GameStarted(GameQuery), //Подія створення нової гри
    GameNewTurn(GameNewTurnQuery), //Подія нового ходу гри
    GameOver(GameOverQuery), //Подія закінчення гри
}