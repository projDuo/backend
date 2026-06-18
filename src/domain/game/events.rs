use uuid::Uuid;

use super::*;

#[derive(Debug, Clone)]
pub enum GameInternalEvents {
    RequestNewGameSession{player_ids: Vec<Uuid>},
    RemovePlayer{ game_id: Uuid, player_id: Uuid },
    ForceTurnIfUnchanged { game_id: Uuid, player_id: Uuid, turn: usize, turn_enforced_at: i64 },
}

#[derive(Debug, Clone)]
pub enum GameEvents {
    GameNewTurn(GameNewTurnQuery), //Подія нового ходу гри
    GameOver(GameOverQuery), //Подія закінчення гри
}