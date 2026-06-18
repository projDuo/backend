use crate::domain::chat::*;
use tokio::sync::broadcast;
use async_trait::async_trait;
use dashmap::DashMap;

type Event = ChatEvent;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

pub struct EventBus {
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
impl ChatEventBus for EventBus {
    fn new_channel(&self, channel_id: String) {
        self.channels.entry(channel_id).or_insert_with(|| Sender::new(self.capacity));
    }

    fn close_channel(&self, channel_id: String) {
        self.channels.remove(&channel_id);
    }

    fn subscribe(&self, channel_id: String) -> Option<Receiver> {
        self.channels
            .get(&channel_id)
            .map(|v| v.subscribe())
    }

    fn publish(&self, channel_id: String, event: Event) {
        let Some(channel) = self.channels.get(&channel_id) else {
            return ();
        };
        let _ = channel.send(event);
    }

    async fn message_posted(&self, message: Message) {
        self.publish(message.channel_id.clone(), ChatEvent::MessagePosted(message));
    }

    async fn message_edited(&self, message: Message) {
        self.publish(message.channel_id.clone(), ChatEvent::MessageEdited(message));
    }

    async fn message_deleted(&self, channel_id: String, id: uuid::Uuid) {
        self.publish(channel_id, ChatEvent::MessageDeleted(id));
    }
}
