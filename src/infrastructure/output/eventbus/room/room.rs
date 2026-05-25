use crate::domain::room::*;
use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use dashmap::DashMap;

type Event = RoomEvent;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

pub struct EventBus{
    channels: DashMap<String, Sender>,
    capacity: usize,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: DashMap::new(),
            capacity,
        }
    }
}

#[async_trait]
impl RoomEventBus for EventBus {
    fn new_channel(&self, room_id: String) {
        self.channels.entry(room_id).or_insert_with(|| Sender::new(self.capacity));
    }
    fn close_channel(&self, room_id: String) {
        self.channels.remove(&room_id);
    }
    fn subscribe(&self, room_id: String) -> Option<Receiver> {
        let res = self.channels.get(&room_id)
            .map(|v| v.subscribe());
    
        res
    }
    fn publish(&self, room_id: String, event: Event) {
        let Some(channel) = self.channels.get(&room_id) else { return () };
        let _ = channel.send(event);
    }
    async fn room_player_new(&self, player: Player) {
        self.publish(player.room_id().clone(), RoomEvent::RoomPlayerNew(player));
    }
    async fn room_player_update(&self, player: Player) {
        self.publish(player.room_id().clone(), RoomEvent::RoomPlayerUpdate(player));
    }
    async fn room_player_left(&self, room_id: String, player_id: Uuid) {
        self.publish(room_id, RoomEvent::RoomPlayerLeft(player_id));
    }
    async fn room_create(&self, room: RoomWithPlayersEmbedded) {
        let id = room.room.id.clone();
        self.publish(id, RoomEvent::RoomCreate(room));
    }
    async fn room_update(&self, room: Room) {
        self.publish(room.id.get().to_string(), RoomEvent::RoomUpdate(room));
    }
    async fn room_delete(&self, room_id: String) {
        self.publish(room_id.clone(), RoomEvent::RoomDelete(room_id));
    }
}