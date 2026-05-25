
use crate::domain::{
    activity::ActivityInternalEventBus, activity::ActivityService, game::GameInternalEventBus,
    room::*, EventBus,
};
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::InternalError;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Service {
    room_repo: Arc<dyn RoomRepository + Send + Sync>,
    player_repo: Arc<dyn PlayerRepository + Send + Sync>,
    event_bus: Arc<dyn RoomEventBus+ Send + Sync>,
    game_internal_event_bus: Arc<dyn GameInternalEventBus + Send + Sync>,
    activity_event_bus: Arc<dyn ActivityInternalEventBus + Send + Sync>,
    activities: Arc<dyn ActivityService + Send + Sync>,
    _shutdown_tx: broadcast::Sender<()>,
}

impl Service {
    pub fn new(
        room_repo: Arc<dyn RoomRepository + Send + Sync>,
        player_repo: Arc<dyn PlayerRepository + Send + Sync>,
        event_bus: Arc<dyn RoomEventBus + Send + Sync>,  
        internal_event_bus: Arc<dyn EventBus<RoomInternalEvent> + Send + Sync>,
        game_internal_event_bus: Arc<dyn GameInternalEventBus + Send + Sync>,
        activity_event_bus: Arc<dyn ActivityInternalEventBus + Send + Sync>,
        activities: Arc<dyn ActivityService + Send + Sync>,
    ) -> Arc<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let service = Arc::new(Self {
            room_repo,
            player_repo,
            event_bus,
            game_internal_event_bus,
            activity_event_bus,
            activities,
            _shutdown_tx: shutdown_tx,
        });

        super::spawn_service_listener(&service, internal_event_bus.subscribe(), shutdown_rx);
                
        service
    }

    pub async fn start_game(&self, room_id: String) -> Result<(), RoomError> {
        let players = self
            .player_repo
            .find_room_players_by_room_id(room_id.clone())
            .await?;

        if !players.iter().all(|p| p.is_ready) {
            return Ok(());
        }

        let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();
        self.game_internal_event_bus
            .request_new_game_session(player_ids)
            .await;

        for player in players {
            let updated = self
                .player_repo
                .update_room_player(UpdatePlayerRequest::new(player.id, Some(false), None))
                .await?;
            self.event_bus.room_player_update(updated).await;
        }

        Ok(())
    }
}

#[async_trait]
impl super::EventHandler<RoomInternalEvent> for Service {
    async fn process(&self, event: RoomInternalEvent) {
        match event {
            RoomInternalEvent::RoomRemovePlayer(v) =>  {
                if let Err(e) = self.remove_room_player(v).await { 
                    tracing::error!("[RoomInternalEvent]Failed to remove room player: {}", e)
                }
            }
        };
    }
}

#[async_trait]
impl RoomService for Service {
    async fn read_room(&self, id: String) -> Result<RoomWithPlayersEmbedded, RoomError> {
        let room = self.room_repo.find_room_by_id(id).await
            .ok_or(RoomError::RoomNotFound)?;
        let players = self.player_repo.find_room_players_with_account_by_room_id(room.id.get().to_string()).await?;
        let res = RoomWithPlayersEmbedded::new(room.into(), players);
        
        Ok(res)
    }

    async fn read_room_list(&self, limit: usize, after: usize) -> Result<Vec<RoomListItem>, InternalError> {
        self.room_repo.read_room_list(limit, after).await
    }

    async fn read_player_by_id(&self, player_id: Uuid) -> Result<Player, RoomError> {
        let player = self.player_repo.find_player_by_id(player_id).await?
            .ok_or(RoomError::PlayerNotInRoom)?;
        Ok(player)
    }

    async fn add_room_player(&self, player_id: Uuid, room_id: String, password: Option<String>) -> Result<Player, RoomError> {
        let room = self.room_repo.find_room_by_id(room_id.clone()).await
            .ok_or(RoomError::RoomNotFound)?;
        let player_count = self.player_repo.room_player_count(room_id.clone()).await?
            .expect("A room shouldn't exist without players.");
        
        let password = password.map(PasswordHash::new_unchecked);
        if password != room.password && room.password.is_some() { 
            return Err(RoomError::WrongPassword)
        }
        
        if player_count >= room.max_players.get() {
            return Err(RoomError::MaxPlayersReached)
        }

        let cmd = CreatePlayerRequest::default_for(player_id, room_id.clone());
        let player = self.player_repo.add_room_player(cmd).await?;

        self.activity_event_bus.user_joined_room(room_id, player_id).await;
        self.event_bus.room_player_new(player.clone()).await;

        Ok(player)
    }

    async fn update_room_player(&self, cmd: UpdatePlayerRequest) -> Result<Player, RoomError> {
        let is_ready_update = cmd.is_ready;

        if is_ready_update.is_some() {
            if let Ok(activity) = self.activities.read_activity(cmd.id).await {
                if activity.game.is_some() {
                    return Err(RoomError::PlayerInGame);
                }
            }
        }

        let player = self.player_repo.update_room_player(cmd).await?;
        self.event_bus.room_player_update(player.clone()).await;

        if let Some(is_ready) = is_ready_update {
            if is_ready == true {
                let _ = self.start_game(player.room_id().clone()).await;
            }
        }

        Ok(player)
    }

    async fn remove_room_player(&self, player_id: Uuid) -> Result<(), RoomError> {
        let player = self.player_repo.find_player_by_id(player_id).await?
            .ok_or(RoomError::PlayerNotInRoom)?;
        let room_id = player.room_id();
        let mut room = self.room_repo.find_room_by_id(room_id.clone()).await
            .expect("A room shouldn't exist while any Player has a reference to it!!!");

        self.player_repo.remove_room_player(player_id).await.map(|_| ())?;
        let player_ids = self.player_repo.room_player_ids(room_id.clone()).await;
        
        self.activity_event_bus.user_left_room(player_id).await;
        self.event_bus.room_player_left(room_id.to_string(), player_id).await;

        if let Some(v) = player_ids.iter().next() {
            if room.owner == player_id {
                room.owner = *v;
            }
        } else {
            self.delete_room(room_id.clone()).await?;
        }
        
        Ok(())
    }

    async fn create_room(&self, cmd: CreateRoomRequest) -> Result<RoomWithPlayersEmbedded, RoomError> {
        if self.player_repo.find_player_by_id(cmd.owner).await?.is_some() {
            return Err(RoomError::PlayerInRoom);
        }

        let room = self.room_repo.create_room(cmd).await?;
        let owner_req = CreatePlayerRequest::default_for(room.owner, room.id.get().to_string());
        self.player_repo.add_room_player(owner_req).await?;
        let players = self.player_repo.find_room_players_with_account_by_room_id(room.id.get().to_string()).await?;
        let res = RoomWithPlayersEmbedded::new(room.into(), players);

        self.event_bus.new_channel(res.room.id.clone());
        self.event_bus.room_create(res.clone()).await;

        Ok(res)
    }
    async fn update_room(&self, cmd: UpdateRoomRequest) -> Result<Room, RoomError> {
        let room = self.room_repo.update_room(cmd).await?;
        self.event_bus.room_update(room.clone()).await;
        Ok(room)
    }
    async fn delete_room(&self, room_id: String) -> Result<(), RoomError> {
        self.room_repo.delete_room(room_id.clone()).await?;
        self.event_bus.room_delete(room_id.clone()).await;
        self.event_bus.close_channel(room_id);
        Ok(())
    }

    async fn get_room_owner(&self, room_id: String) -> Result<Uuid, RoomError> {
        self.room_repo.get_room_owner(room_id).await
    }
}