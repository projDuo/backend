use serde::Serialize;
use structural_convert::StructuralConvert;
use uuid::Uuid;
use crate::domain::activity::events::ActivityEvent;

pub mod output;

#[derive(Debug, Clone, StructuralConvert, Serialize)]
#[convert(from(ActivityEvent))]
#[serde(tag = "event", content = "data")]
pub enum ActivityEventOut {
    JoinedRoom{room_id: String},
    LeftRoom,
    JoinedGame{game_id: Uuid},
    LeftGame,
}