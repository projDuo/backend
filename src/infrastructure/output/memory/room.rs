use std::{collections::{HashMap, hash_map::Entry}};
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;
use crate::domain::{accounts::{AccountsRepository}, room::*};
use indexmap::IndexSet;
use dashmap::DashMap;
use crate::domain::InternalError;

pub struct RoomStorage{
    accounts: Arc<dyn AccountsRepository + Send + Sync>,
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    players: DashMap<Uuid, Player>,
    occupancy: DashMap<String, IndexSet<Uuid>>,
}

impl RoomStorage {
    pub fn new(accounts_repo: Arc<dyn AccountsRepository + Send + Sync>) -> Self {
        let rooms = Arc::new(
            RwLock::new(HashMap::new())
        );
        let players = DashMap::new();
        let occupancy = DashMap::new();
        Self { accounts: accounts_repo, rooms, players, occupancy }
    }
}

#[async_trait]
impl RoomRepository for RoomStorage {
    async fn find_room_by_id(&self, id: String) -> Option<Room> {
        let storage = self.rooms.read().await;
        storage.get(&id).map(Clone::clone)
    }
    async fn read_room_list(&self, limit: usize, after: usize) -> Result<Vec<RoomListItem>, InternalError> {
        let storage = self.rooms.read().await;
        let rooms: Vec<Room> = storage
            .values()
            .filter(|v| *v.is_public.get())
            .skip(after)
            .take(limit)
            .cloned()
            .collect();
        let mut room_ids = Vec::new();
        let mut owner_ids = Vec::new();

        for room in rooms.iter().rev() {
            room_ids.push(room.id.get().to_string());
            owner_ids.push(room.owner.clone());
        }

        let (accounts_attempt, mut player_counts) = tokio::join!(
            self.accounts.find_many_accounts_by_id_public(owner_ids),
            async {
                let player_counts: Vec<u8> = room_ids
                    .iter()
                    .filter_map(|id| {
                        self.occupancy.get(id).map(|v| v.len() as u8)
                    })
                    .collect();

                player_counts
            }
        );

        let mut accounts = accounts_attempt?;

        let res = rooms
            .iter()
            .map(|room| {
                RoomListItem::new(
                    room.id.get().to_string(),
                    room.name.get().to_string(),
                    *room.is_public.get(),
                    room.password.is_some(),
                    accounts.pop().expect("Room and owner id vectors size mismatch"),
                    room.max_players.get(),
                    player_counts.pop().expect("Room and player count vectors size mismatch")
                )
            })
            .collect();

        Ok(res)
    }
    async fn create_room(&self, cmd: CreateRoomRequest) -> Result<Room, RoomError> {
        let mut storage = self.rooms.write().await;

        let name = cmd.name.unwrap_or_default();
        let is_public: IsPublic = cmd.is_public.unwrap_or_default();
        let password = cmd.password.unwrap_or_default();
        let max_players = cmd.max_players.unwrap_or_default();
        let res = loop {
            let id = RoomId::generate();
            let entry = storage.entry(id.get().clone());
            if let Entry::Occupied(_) = entry { continue }
            let room = Room::new(id, name, is_public, password, cmd.owner, max_players);
            entry.insert_entry(room.clone());
            break room
        };

        Ok(res)
    }
    async fn update_room(&self, cmd: UpdateRoomRequest) -> Result<Room, RoomError> {
        let mut storage = self.rooms.write().await;
        let room = storage.get_mut(&cmd.id)
            .ok_or(RoomError::RoomNotFound)?;

        if let Some(v) = cmd.name { room.name = v };
        if let Some(v) = cmd.is_public { room.is_public = v };
        if let Some(v) = cmd.password { room.password = v };
        if let Some(v) = cmd.owner { room.owner = v };
        if let Some(v) = cmd.max_players { room.max_players = v };
        
        let res = room.clone();
        Ok(res)
    }
    async fn delete_room(&self, id: String) -> Result<(), RoomError> {
        let mut storage = self.rooms.write().await;
        storage.remove(&id)
            .ok_or(RoomError::RoomNotFound)?;

        Ok(())
    }

    async fn get_room_owner(&self, room_id: String) -> Result<Uuid, RoomError> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id)
            .ok_or(RoomError::RoomNotFound)?;
        let owner = room.owner;
        Ok(owner)
    }
}

#[async_trait]
impl PlayerRepository for RoomStorage {
    async fn find_room_players_by_room_id(&self, id: String) -> Result<Vec<Player>, InternalError> {
        let res = self.occupancy
            .get(&id)
            .map(|set| {
                set.iter()
                    .filter_map(|player_id| self.players.get(player_id))
                    .map(|p| p.clone())
                    .collect()
            })
            .unwrap_or_default();

        Ok(res)
    }
    async fn find_room_players_with_account_by_room_id(&self, id: String) -> Result<Vec<PlayerWithAccount>, InternalError> {
        let players = self.find_room_players_by_room_id(id).await?;
        let ids: Vec<Uuid> = players.iter().rev().map(|v| v.id.clone()).collect();
        let mut accounts = self.accounts.find_many_accounts_by_id_public(ids).await?;

        let players_with_accounts: Vec<PlayerWithAccount> = players.into_iter().map(|v| 
            PlayerWithAccount::new(
                accounts.pop().expect("Players and accounts vectors length mismatch"),
                v.into()
            )
        ).collect();

        Ok(players_with_accounts)
    }
    async fn find_player_by_id(&self, id: Uuid) -> Result<Option<Player>, InternalError> {
        let player = self.players.get(&id)
            .map(|v| v.clone());
        Ok(player)
    }
    async fn add_room_player(&self, cmd: CreatePlayerRequest) -> Result<Player, InternalError> {
        //Remove player from room occupancy map in case
        //the player is forced to join another room
        if let Some(p) = self.players.get(&cmd.id) {
            if let Some(mut r) = self.occupancy.get_mut(p.room_id()) {
                r.shift_remove(&p.id);
            }
        }
        
        let player = Player::new(cmd.id, cmd.room_id.to_string(), cmd.is_ready, cmd.points);
        self.occupancy.entry(cmd.room_id.to_string()).or_default().insert(cmd.id);
        //If player was already in a room their data will be replaced
        self.players.insert(cmd.id, player.clone());
        
        Ok(player)
    }
    async fn update_room_player(&self, cmd: UpdatePlayerRequest) -> Result<Player, RoomError> {
        let mut player = self.players.get_mut(&cmd.id)
            .ok_or(RoomError::PlayerNotFound)?;
        
        if let Some(v) = cmd.is_ready {
            player.is_ready = v;
        }
        if let Some(v) = cmd.points {
            player.points = v;
        }
        Ok(player.clone())
    }
    async fn remove_room_player(&self, player_id: Uuid) -> Result<(), RoomError> {
        let room_id = self.players.get(&player_id)
            .ok_or(RoomError::PlayerNotFound)?
            .room_id()
            .clone();
        if let Some(mut v) = self.occupancy.get_mut(&room_id) {
            v.shift_remove(&player_id);
        }
        self.players.remove(&player_id);
        
        Ok(())
    }
    async fn remove_all_room_players(&self, room_id: String) -> Result<(), RoomError> {
        for player_id in self.occupancy.entry(room_id.to_string()).or_default().iter() {
            self.players.remove(player_id);
        }
        
        self.occupancy.remove(&room_id);
        Ok(())
    }

    async fn room_player_ids(&self, room_id: String) -> Vec<Uuid> {
        self.occupancy.entry(room_id.to_string()).or_default().iter().cloned().collect()
    }

    async fn room_player_count(&self, room_id: String) -> Result<Option<u8>, InternalError> {
        let player_count = self.occupancy.get(&room_id).map(|v| v.len() as u8);
        Ok(player_count)
    }
}