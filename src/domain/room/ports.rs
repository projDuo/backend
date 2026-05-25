use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::domain::room::RoomEvent;

use super::entities::*;
use super::errors::*;
use super::commands::*;
use super::query::*;
use super::InternalError;

#[async_trait]
pub trait RoomRepository {
    async fn find_room_by_id(&self, id: String) -> Option<Room>;
    async fn read_room_list(&self, limit: usize, after: usize) -> Result<Vec<RoomListItem>, InternalError>;
    async fn create_room(&self, cmd: CreateRoomRequest) -> Result<Room, RoomError>;
    async fn update_room(&self, cmd: UpdateRoomRequest) -> Result<Room, RoomError>;
    async fn delete_room(&self, id: String) -> Result<(), RoomError>;
    async fn get_room_owner(&self, id: String) -> Result<Uuid, RoomError>;
}

#[async_trait]
pub trait PlayerRepository {
    async fn find_room_players_by_room_id(&self, id: String) -> Result<Vec<Player>, InternalError>;
    async fn find_room_players_with_account_by_room_id(&self, id: String) -> Result<Vec<PlayerWithAccount>, InternalError>;
    async fn find_player_by_id(&self, id: Uuid) -> Result<Option<Player>, InternalError>;
    async fn add_room_player(&self, cmd: CreatePlayerRequest) -> Result<Player, InternalError>;
    async fn update_room_player(&self, cmd: UpdatePlayerRequest) -> Result<Player, RoomError>;
    async fn remove_room_player(&self, player_id: Uuid) -> Result<(), RoomError>;
    async fn remove_all_room_players(&self, room_id: String) -> Result<(), RoomError>;
    async fn room_player_ids(&self, room_id: String) -> Vec<Uuid>;
    async fn room_player_count(&self, room_id: String) -> Result<Option<u8>, InternalError>;
}

#[async_trait]
pub trait RoomService {
    async fn read_room(&self, id: String) -> Result<RoomWithPlayersEmbedded, RoomError>;
    async fn read_room_list(&self, limit: usize, after: usize) -> Result<Vec<RoomListItem>, InternalError>;
    async fn read_player_by_id(&self, player_id: Uuid) -> Result<Player, RoomError>;
    async fn add_room_player(&self, player_id: Uuid, room_id: String, password: Option<String>) -> Result<Player, RoomError>;
    async fn update_room_player(&self, cmd: UpdatePlayerRequest) -> Result<Player, RoomError>;
    async fn remove_room_player(&self, player_id: Uuid) -> Result<(), RoomError>;
    async fn create_room(&self, cmd: CreateRoomRequest) -> Result<RoomWithPlayersEmbedded, RoomError>;
    async fn update_room(&self, cmd: UpdateRoomRequest) -> Result<Room, RoomError>;
    async fn delete_room(&self, room_id: String) -> Result<(), RoomError>;
    async fn get_room_owner(&self, room_id: String) -> Result<Uuid, RoomError>;
}

#[async_trait]
pub trait RoomEventBus: Send + Sync {
    fn new_channel(&self, room_id: String);
    fn close_channel(&self, room_id: String);
    fn subscribe(&self, room_id: String) -> Option<broadcast::Receiver<RoomEvent>>;
    fn publish(&self, room_id: String, event: RoomEvent);
    async fn room_player_new(&self, player: Player);
    async fn room_player_update(&self, player: Player);
    async fn room_player_left(&self, room_id: String, player_id: Uuid);
    async fn room_create(&self, room: RoomWithPlayersEmbedded);
    async fn room_update(&self, room: Room);
    async fn room_delete(&self, room_id: String);
}

#[async_trait]
pub trait RoomInternalEventBus {
    async fn remove_room_player(&self, player_id: Uuid);
}
