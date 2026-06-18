use uuid::Uuid;
use serde::Serialize;
use structural_convert::StructuralConvert;
use crate::domain::room::{ self, entities, query };

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(entities::Player))]
pub struct Player {
    pub id: Uuid,
    pub is_ready: bool, 
}

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(query::PlayerBody))]
pub struct PlayerBodyResponse {
    pub is_ready: bool, 
}

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(query::PlayerWithAccount))]
pub struct PlayerWithAccountResponse {
    #[serde(flatten)]
    pub account: super::super::account::output::AccountReadPublic,
    #[serde(flatten)]
    pub player: PlayerBodyResponse,
}

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(entities::Player))]
pub struct PlayerReady {
    pub is_ready: bool,
}

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(room::query::RoomPublic))]
pub struct RoomPublicResponse {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub password: bool,
    pub owner: Uuid,
    pub max_players: u8,
}

impl From<entities::Room> for RoomPublicResponse {
    fn from(value: entities::Room) -> Self {
        Self {
            id: value.id.get().to_string(),
            name: value.name.get().to_string(),
            is_public: value.is_public,
            password: value.password.is_some(),
            owner: value.owner,
            max_players: value.max_players.get()
        }
    }
}

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(query::RoomWithPlayersEmbedded))]
pub struct RoomWithPlayersEmbeddedResponse {
    pub room: RoomPublicResponse,
    pub players: Vec<PlayerWithAccountResponse>,
}