use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::EventBus as EventBusTrait;
use super::super::impl_tokio_event_bus;
use crate::domain::game::*;

type Event = GameInternalEvents;
type Sender = broadcast::Sender<Event>;
type Receiver = broadcast::Receiver<Event>;

impl_tokio_event_bus!(
    InternalEventBus,
    Sender,
    Receiver,
    Event
);

#[async_trait]
impl GameInternalEventBus for InternalEventBus {
    async fn request_new_game_session(&self, player_ids: Vec<Uuid>) {
        self.publish(GameInternalEvents::RequestNewGameSession { player_ids });
    }
    async fn remove_player(&self, game_id: Uuid, player_id: Uuid) {
        self.publish(GameInternalEvents::RemovePlayer { game_id, player_id });
    }
}