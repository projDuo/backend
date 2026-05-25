use uuid::Uuid;
use async_trait::async_trait;
use crate::domain::activity::events::ActivityEvent;
use tokio::sync::broadcast;

use super::*;

#[async_trait]
pub trait ActivityService {
    async fn mark_activity(&self, cmd: MarkActivityCommand) -> Result<Activity, ActivityError>;
    async fn read_activity(&self, user_id: Uuid) -> Result<Activity, ActivityError>;
    async fn delete_activity(&self, user_id: Uuid) -> Result<(), ActivityError>;
}

#[async_trait]
pub trait ActivityInternalEventBus {
    async fn user_joined_room(&self, room_id: String, user_id: Uuid);
    async fn user_left_room(&self, user_id: Uuid);
    async fn user_joined_game(&self, game_id: Uuid, user_id: Uuid);
    async fn user_left_game(&self, user_id: Uuid);
}

#[async_trait]
pub trait ActivityEventBus {
    fn new_channel(&self, user_id: Uuid);
    fn close_channel(&self, user_id: Uuid);
    fn subscribe(&self, user_id: Uuid) -> Option<broadcast::Receiver<ActivityEvent>>;
    fn publish(&self, user_id: Uuid, event: ActivityEvent);
    async fn joined_room(&self, room_id: String, user_id: Uuid);
    async fn left_room(&self, user_id: Uuid);
    async fn joined_game(&self, game_id: Uuid, user_id: Uuid);
    async fn left_game(&self, user_id: Uuid);
}