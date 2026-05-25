use super::entities::*;

use uuid::Uuid;

pub struct CreateRoomRequest {
    pub name: Option<Name>,
    pub is_public: Option<IsPublic>,
    pub password: Option<Option<PasswordHash>>,
    pub owner: Uuid,
    pub max_players: Option<MaxPlayers>,
}
impl CreateRoomRequest {
    pub fn new(
        name: Option<Name>,
        is_public: Option<IsPublic>,
        password: Option<Option<PasswordHash>>,
        owner: Uuid,
        max_players: Option<MaxPlayers>,
    ) -> Self {
        Self { name, is_public, password, owner, max_players }
    }
}

pub struct UpdateRoomRequest {
    pub id: String,
    pub name: Option<Name>,
    pub is_public: Option<IsPublic>,
    pub password: Option<Option<PasswordHash>>,
    pub owner: Option<Uuid>,
    pub max_players: Option<MaxPlayers>,
}
impl UpdateRoomRequest {
    pub fn new(
        id: String,
        name: Option<Name>,
        is_public: Option<IsPublic>,
        password: Option<Option<PasswordHash>>,
        owner: Option<Uuid>,
        max_players: Option<MaxPlayers>,
    ) -> Self {
        Self { id, name, is_public, password, owner, max_players }
    }
}

pub struct CreatePlayerRequest {
    pub id: Uuid,
    pub room_id: String,
    pub is_ready: bool,
    pub points: u64,
}
impl CreatePlayerRequest {
    pub fn default_for(player_id: Uuid, room_id: String) -> Self {
        Self { id: player_id, room_id, is_ready: false, points: 0 }
    }
}

pub struct UpdatePlayerRequest {
    pub id: Uuid,
    pub is_ready: Option<bool>,
    pub points: Option<u64>,
}
impl UpdatePlayerRequest {
    pub fn new(id: Uuid, is_ready: Option<bool>, points: Option<u64>) -> Self {
        Self { id, is_ready, points }
    }
}
impl From<Player> for UpdatePlayerRequest {
    fn from(value: Player) -> Self {
        Self::new(
            value.id,
            Some(value.is_ready),
            Some(value.points)
        )
    }
}