use crate::domain::room::*;
use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::EventBus as EventBusTrait;

type Event = RoomInternalEvent;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

super::impl_tokio_event_bus!(
    EventBus,
    Sender,
    Receiver,
    Event
);

#[async_trait]
impl RoomInternalEventBus for EventBus {
    async fn remove_room_player(&self, player_id: Uuid) {
        self.publish(RoomInternalEvent::RoomRemovePlayer(player_id));
    }
}