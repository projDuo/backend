use crate::domain::activity::{ActivityEventBus, events::ActivityEvent};
use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use dashmap::DashMap;

type Event = ActivityEvent;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

pub struct EventBus{
    channels: DashMap<Uuid, Sender>,
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
impl ActivityEventBus for EventBus {
    fn new_channel(&self, user_id: Uuid) {
        self.channels.entry(user_id).or_insert_with(|| Sender::new(self.capacity));
    }
    fn close_channel(&self, user_id: Uuid) {
        self.channels.remove(&user_id);
    }
    fn subscribe(&self, user_id: Uuid) -> Option<Receiver> {
        let res = self.channels.get(&user_id)
            .map(|v| v.subscribe());
    
        res
    }
    fn publish(&self, user_id: Uuid, event: Event) {
        let Some(channel) = self.channels.get(&user_id) else { return () };
        let _ = channel.send(event);
    }
    async fn joined_room(&self, room_id: String, user_id: Uuid) {
        self.publish(user_id, ActivityEvent::JoinedRoom { room_id });
    }
    async fn left_room(&self, user_id: Uuid) {
        self.publish(user_id, ActivityEvent::LeftRoom);
    }
    async fn joined_game(&self, game_id: Uuid, user_id: Uuid) {
        self.publish(user_id, ActivityEvent::JoinedGame { game_id });
    }
    async fn left_game(&self, user_id: Uuid) {
        self.publish(user_id, ActivityEvent::LeftGame);
    }
}