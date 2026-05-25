use crate::{domain::activity::{events::ActivityInternalEvent, *}, impl_tokio_event_bus};
use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::EventBus as EventBusTrait;

type Event = ActivityInternalEvent;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

impl_tokio_event_bus!(EventBus, Sender, Receiver, Event);
#[async_trait]
impl ActivityInternalEventBus for EventBus {
    async fn user_joined_room(&self, room_id: String, user_id: Uuid) {
        self.publish(ActivityInternalEvent::UserJoinedRoom{room_id, user_id});
    }
    async fn user_left_room(&self, user_id: Uuid) {
        self.publish(ActivityInternalEvent::UserLeftRoom{user_id});
    }
    async fn user_joined_game(&self, game_id: Uuid, user_id: Uuid) {
        tracing::info!("why: {} {}", game_id, user_id);
        self.publish(ActivityInternalEvent::UserJoinedGame{game_id, user_id});
    }
    async fn user_left_game(&self, user_id: Uuid) {
        self.publish(ActivityInternalEvent::UserLeftGame{user_id});
    }
}