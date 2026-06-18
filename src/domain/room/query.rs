use uuid::Uuid;

use crate::domain::accounts::query::AccountPublic;
use super::entities::*;

pub struct RoomListItem {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub password: bool,
    pub owner: AccountPublic,
    pub max_players: u8,
    pub players: u8,
}
impl RoomListItem {
    pub fn new(
        id: String,
        name: String,
        is_public: bool,
        password: bool,
        owner: AccountPublic,
        max_players: u8,
        players: u8,
    ) -> Self {
        Self { id, name, is_public, password, owner, max_players, players }
    }
}

#[derive(Debug, Clone)]
pub struct RoomPublic {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub password: bool,
    pub owner: Uuid,
    pub max_players: u8,
}
impl From<Room> for RoomPublic {
    fn from(value: Room) -> Self {
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

#[derive(Debug, Clone)]
pub struct PlayerBody {
    pub is_ready: bool, 
}
impl From<Player> for PlayerBody {
    fn from(value: Player) -> Self {
        Self { is_ready: value.is_ready }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerWithAccount {
    pub account: AccountPublic,
    pub player: PlayerBody,
}
impl PlayerWithAccount {
    pub fn new(
        account: AccountPublic,
        player: PlayerBody,
    ) -> Self {
        Self {
            account,
            player
        }
    }
}


#[derive(Debug, Clone)]
pub struct RoomWithPlayersEmbedded {
    pub room: RoomPublic,
    pub players: Vec<PlayerWithAccount>,
}
impl RoomWithPlayersEmbedded {
    pub fn new(room: RoomPublic, players: Vec<PlayerWithAccount>) -> Self {
        Self {
            room,
            players
        }
    }
}