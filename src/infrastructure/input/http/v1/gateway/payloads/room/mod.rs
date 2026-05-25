use structural_convert::StructuralConvert;
use uuid::Uuid;
use serde::Serialize;

use crate::domain::room::RoomEvent;

pub mod output;

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[serde(tag = "event", content = "data")]
#[convert(from(RoomEvent))]
pub enum RoomEventOut {
    RoomPlayerNew(output::Player),
    RoomPlayerUpdate(output::Player),
    RoomPlayerLeft(Uuid),
    RoomCreate(output::RoomWithPlayersEmbeddedResponse),
    RoomUpdate(output::RoomPublicResponse),
    RoomDelete(String),
}