use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::InternalError;
use super::entities::*;
use super::errors::*;

#[async_trait]
pub trait GameHistoryRepository {
    async fn insert_history(&self, cmd: GameHistory) -> Result<GameHistory, GameHistoryError>;
    async fn list_by_account(&self, account_id: Uuid, after: Option<Uuid>, limit: Option<u32>) -> Result<Vec<GameHistory>, InternalError>;
}

#[async_trait]
pub trait GameHistoryService {
    async fn record_history(&self, cmd: GameHistory) -> Result<GameHistory, GameHistoryError>;
    async fn list_player_history(&self, account_id: Uuid, after: Option<Uuid>, limit: Option<u32>) -> Result<Vec<GameHistory>, InternalError>;
}
