use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;
use super::errors::*;
use crate::domain::InternalError;

#[async_trait]
pub trait MutedRepository {
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError>;
    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError>;
    async fn is_user_muted(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, InternalError>;
    async fn get_muted_users(&self, blocker_id: Uuid) -> Result<Vec<MutedEntry>, InternalError>;
}

#[async_trait]
pub trait MutedService {
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), MutedError>;
    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), MutedError>;
    async fn is_user_muted(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, MutedError>;
    async fn get_muted_users(&self, blocker_id: Uuid) -> Result<Vec<MutedEntry>, MutedError>;
}
