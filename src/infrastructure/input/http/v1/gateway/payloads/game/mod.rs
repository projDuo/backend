use serde::Serialize;
use structural_convert::StructuralConvert;

use crate::domain::game::GameEvents;

pub mod output;

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[serde(tag = "event", content = "data")]
#[convert(from(GameEvents))]
pub enum GameEventOut {
    GameNewTurn(output::GameNewTurnQuery), //Подія нового ходу гри
    GameOver(output::GameOverQuery), //Подія закінчення гри
}