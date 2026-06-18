
use crate::domain::{
    activity::ActivityInternalEventBus, activity::ActivityService, game::GameInternalEventBus,
    room::*, chat::{ChatService, ChatEventBus}, EventBus,
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
    chat: Arc<dyn ChatService + Send + Sync>,
    chat_event_bus: Arc<dyn ChatEventBus + Send + Sync>,
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
        chat: Arc<dyn ChatService + Send + Sync>,
        chat_event_bus: Arc<dyn ChatEventBus + Send + Sync>,
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
            chat,
            chat_event_bus,
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
                .update_room_player(UpdatePlayerRequest::new(player.id, Some(false)))
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

    async fn read_room_list(&self, limit: usize, after: usize, search: Option<String>) -> Result<Vec<RoomListItem>, InternalError> {
        self.room_repo.read_room_list(limit, after, search).await
    }

    async fn read_player_by_id(&self, player_id: Uuid) -> Result<Player, RoomError> {
        let player = self.player_repo.find_player_by_id(player_id).await?
            .ok_or(RoomError::PlayerNotInRoom)?;
        Ok(player)
    }

    async fn kick_room_player(&self, initiator_id: Uuid, room_id: String, player_id: Uuid) -> Result<(), RoomError> {
        let room_owner = self.room_repo.get_room_owner(room_id.clone()).await?;
        if room_owner != initiator_id {
            return Err(RoomError::NotAnOwner);
        }

        let target_player = self.player_repo.find_player_by_id(player_id).await?
            .ok_or(RoomError::PlayerNotFound)?;
        if target_player.room_id() != &room_id {
            return Err(RoomError::PlayerNotFound);
        }

        self.remove_room_player(player_id).await
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

        self.activity_event_bus.user_joined_room(room_id.clone(), player_id).await;
        let _ = self.chat.add_member(player_id, room_id).await;
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
        
        let _ = self.chat.remove_member(player_id, room_id.as_str()).await;
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

        let room_id = res.room.id.clone();
        self.event_bus.new_channel(room_id.clone());
        self.chat.create_channel(room_id.clone()).await;
        let _ = self.chat.add_member(res.room.owner, room_id.clone()).await;
        self.chat_event_bus.new_channel(room_id);
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
        self.event_bus.close_channel(room_id.clone());
        self.chat.delete_channel(&room_id).await;
        self.chat_event_bus.close_channel(room_id);
        Ok(())
    }

    async fn get_room_owner(&self, room_id: String) -> Result<Uuid, RoomError> {
        self.room_repo.get_room_owner(room_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activity::{Activity, ActivityError, MarkActivityCommand};
    use crate::domain::accounts::AccountPublic;
    use crate::domain::chat::{ChatError, ChatEvent, CreateMessageCommand, DeleteMessageCommand, Message, UpdateMessageCommand};
    use crate::domain::InternalError;
    use crate::domain::room::PlayerBody;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use uuid::Uuid;

    struct DummyRoomRepo {
        room: Room,
    }

    #[async_trait]
    impl RoomRepository for DummyRoomRepo {
        async fn find_room_by_id(&self, _id: String) -> Option<Room> {
            Some(self.room.clone())
        }
        async fn read_room_list(&self, _limit: usize, _after: usize, _search: Option<String>) -> Result<Vec<RoomListItem>, InternalError> { Ok(vec![]) }
        async fn create_room(&self, _cmd: CreateRoomRequest) -> Result<Room, RoomError> { unimplemented!() }
        async fn update_room(&self, _cmd: UpdateRoomRequest) -> Result<Room, RoomError> { unimplemented!() }
        async fn delete_room(&self, _id: String) -> Result<(), RoomError> { unimplemented!() }
        async fn get_room_owner(&self, _id: String) -> Result<Uuid, RoomError> { Ok(self.room.owner) }
    }

    struct DummyPlayerRepo {
        player: Player,
        players: Vec<PlayerWithAccount>,
    }

    #[async_trait]
    impl PlayerRepository for DummyPlayerRepo {
        async fn find_room_players_by_room_id(&self, _id: String) -> Result<Vec<Player>, InternalError> { Ok(vec![self.player.clone()]) }
        async fn find_room_players_with_account_by_room_id(&self, _id: String) -> Result<Vec<PlayerWithAccount>, InternalError> { Ok(self.players.clone()) }
        async fn find_player_by_id(&self, _id: Uuid) -> Result<Option<Player>, InternalError> { Ok(Some(self.player.clone())) }
        async fn add_room_player(&self, _cmd: CreatePlayerRequest) -> Result<Player, InternalError> { Ok(self.player.clone()) }
        async fn update_room_player(&self, _cmd: UpdatePlayerRequest) -> Result<Player, RoomError> { Ok(self.player.clone()) }
        async fn remove_room_player(&self, _player_id: Uuid) -> Result<(), RoomError> { Ok(()) }
        async fn remove_all_room_players(&self, _room_id: String) -> Result<(), RoomError> { Ok(()) }
        async fn room_player_ids(&self, _room_id: String) -> Vec<Uuid> { vec![self.player.id] }
        async fn room_player_count(&self, _room_id: String) -> Result<Option<u8>, InternalError> { Ok(Some(1)) }
    }

    struct DummyRoomEventBus;
    #[async_trait]
    impl RoomEventBus for DummyRoomEventBus {
        fn new_channel(&self, _room_id: String) {}
        fn close_channel(&self, _room_id: String) {}
        fn subscribe(&self, _room_id: String) -> Option<broadcast::Receiver<RoomEvent>> { None }
        fn publish(&self, _room_id: String, _event: RoomEvent) {}
        async fn room_player_new(&self, _player: Player) {}
        async fn room_player_update(&self, _player: Player) {}
        async fn room_player_left(&self, _room_id: String, _player_id: Uuid) {}
        async fn room_create(&self, _room: RoomWithPlayersEmbedded) {}
        async fn room_update(&self, _room: Room) {}
        async fn room_delete(&self, _room_id: String) {}
    }

    struct DummyChatService;
    #[async_trait]
    impl ChatService for DummyChatService {
        async fn channel_exists(&self, _channel_id: &str) -> bool { false }
        async fn is_member(&self, _user_id: Uuid, _channel_id: &str) -> bool { false }
        async fn create_channel(&self, _channel_id: String) {}
        async fn delete_channel(&self, _channel_id: &str) {}
        async fn add_member(&self, _user_id: Uuid, _channel_id: String) -> Result<(), InternalError> { Ok(()) }
        async fn remove_member(&self, _user_id: Uuid, _channel_id: &str) -> Result<(), InternalError> { Ok(()) }
        async fn read_messages(&self, _user_id: Uuid, _channel_id: String, _after: usize, _limit: usize) -> Result<Vec<Message>, ChatError> { unimplemented!() }
        async fn post_message(&self, _cmd: CreateMessageCommand) -> Result<Message, ChatError> { unimplemented!() }
        async fn edit_message(&self, _cmd: UpdateMessageCommand) -> Result<Message, ChatError> { unimplemented!() }
        async fn delete_message(&self, _cmd: DeleteMessageCommand) -> Result<(), ChatError> { unimplemented!() }
    }

    struct DummyChatEventBus;
    #[async_trait]
    impl ChatEventBus for DummyChatEventBus {
        fn new_channel(&self, _channel_id: String) {}
        fn close_channel(&self, _channel_id: String) {}
        fn subscribe(&self, _channel_id: String) -> Option<broadcast::Receiver<ChatEvent>> { None }
        fn publish(&self, _channel_id: String, _event: ChatEvent) {}
        async fn message_posted(&self, _message: Message) {}
        async fn message_edited(&self, _message: Message) {}
        async fn message_deleted(&self, _channel_id: String, _id: Uuid) {}
    }

    struct DummyInternalEventBus;
    impl<T: Clone + Send + Sync + 'static> EventBus<T> for DummyInternalEventBus {
        fn subscribe(&self) -> broadcast::Receiver<T> {
            let (sender, _) = broadcast::channel(1);
            sender.subscribe()
        }
        fn publish(&self, _event: T) {}
    }

    struct DummyGameInternalEventBus;
    #[async_trait]
    impl GameInternalEventBus for DummyGameInternalEventBus {
        async fn request_new_game_session(&self, _player_ids: Vec<Uuid>) {}
        async fn remove_player(&self, _game_id: Uuid, _player_id: Uuid) {}
    }

    struct DummyActivityInternalEventBus;
    #[async_trait]
    impl ActivityInternalEventBus for DummyActivityInternalEventBus {
        async fn user_joined_room(&self, _room_id: String, _user_id: Uuid) {}
        async fn user_left_room(&self, _user_id: Uuid) {}
        async fn user_joined_game(&self, _game_id: Uuid, _user_id: Uuid) {}
        async fn user_left_game(&self, _user_id: Uuid) {}
    }

    struct DummyActivityService;
    #[async_trait]
    impl ActivityService for DummyActivityService {
        async fn mark_activity(&self, _cmd: MarkActivityCommand) -> Result<Activity, ActivityError> { unimplemented!() }
        async fn read_activity(&self, _user_id: Uuid) -> Result<Activity, ActivityError> { Err(ActivityError::NotFound) }
        async fn delete_activity(&self, _user_id: Uuid) -> Result<(), ActivityError> { Ok(()) }
    }

    #[tokio::test]
    async fn read_room_returns_room_with_players() {
        let room_id = RoomId::generate();
        let owner = Uuid::now_v7();
        let room = Room::new(
            room_id.clone(),
            Name::new("Test room".to_string()).unwrap(),
            true,
            None,
            owner,
            MaxPlayers::new(2).unwrap(),
        );
        let player_id = Uuid::now_v7();
        let player = Player::new(player_id, room_id.get().to_string(), false);
        let account = AccountPublic { id: player_id, login: "player".to_string(), display_name: None, created_at: chrono::Utc::now().into() };
        let player_with_account = PlayerWithAccount::new(account, PlayerBody::from(player.clone()));

        let room_repo = Arc::new(DummyRoomRepo { room: room.clone() });
        let player_repo = Arc::new(DummyPlayerRepo { player: player.clone(), players: vec![player_with_account.clone()] });

        let service = Service::new(
            room_repo,
            player_repo,
            Arc::new(DummyRoomEventBus),
            Arc::new(DummyChatService),
            Arc::new(DummyChatEventBus),
            Arc::new(DummyInternalEventBus),
            Arc::new(DummyGameInternalEventBus),
            Arc::new(DummyActivityInternalEventBus),
            Arc::new(DummyActivityService),
        );

        let result = service.read_room(room_id.get().to_string()).await.expect("read room");
        assert_eq!(result.room.id, room_id.get().to_string());
        assert_eq!(result.players.len(), 1);
    }
}
