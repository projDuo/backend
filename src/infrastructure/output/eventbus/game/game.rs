use crate::domain::game::*;
use tokio::sync::broadcast;
use async_trait::async_trait;
use uuid::Uuid;
use dashmap::DashMap;

type Event = GameEvents;
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
impl GameEventBus for EventBus {
    fn new_channel(&self, player_id: Uuid) {
        self.channels.entry(player_id).or_insert_with(|| Sender::new(self.capacity));
    }
    fn close_channel(&self, player_id: Uuid) {
        self.channels.remove(&player_id);
    }
    fn subscribe(&self, player_id: Uuid) -> Option<Receiver> {
        let res = self.channels.get(&player_id)
            .map(|v| v.subscribe());
    
        res
    }
    fn publish(&self, player_id: Uuid, event: Event) {
        let Some(channel) = self.channels.get(&player_id) else { return () };
        let _ = channel.send(event);
    }
    async fn game_new_turn(&self, player_id: Uuid, game: GameNewTurnQuery) {
        self.publish(player_id, GameEvents::GameNewTurn(game));
    }
    async fn game_over(&self, player_id: Uuid, game: GameOverQuery) {
        self.publish(player_id, GameEvents::GameOver(game));
    }
}