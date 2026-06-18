use async_trait::async_trait;
use uuid::Uuid;
use tokio::sync::broadcast;

use super::*;

#[async_trait]
pub trait GameService {
    async fn create_game_session(&self, initiator_player_id: Option<Uuid>, player_ids: Vec<Uuid>) -> Result<GameQuery, GameError>;
    async fn read_game_session_as_player(&self, player_id: Uuid, game_id: Uuid) -> Result<GameQuery, GameError>;
    async fn play_card(&self, game_id: Uuid, player_id: Uuid, card_id: Option<usize>) -> Result<PlayCard, GameError>;
    async fn remove_player(&self, game_id: Uuid, player_id: Uuid) -> Result<(), GameError>;
}

#[async_trait]
pub trait GameInternalEventBus {
    async fn request_new_game_session(&self, player_ids: Vec<Uuid>);
    async fn remove_player(&self, game_id: Uuid, player_id: Uuid);
}

#[async_trait]
pub trait GameEventBus {
    fn new_channel(&self, player_id: Uuid);
    fn close_channel(&self, player_id: Uuid);
    fn subscribe(&self, player_id: Uuid) -> Option<broadcast::Receiver<GameEvents>>;
    fn publish(&self, player_id: Uuid, event: GameEvents);
    async fn game_new_turn(&self, player_id: Uuid, game: GameNewTurnQuery);
    async fn game_over(&self, player_id: Uuid, game: GameOverQuery);
}