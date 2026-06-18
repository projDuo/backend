use async_trait::async_trait;
use uuid::Uuid;
use tokio::sync::broadcast;

use super::entities::*;
use super::errors::*;
use super::commands::*;
use super::events::*;
use super::InternalError;

#[async_trait]
pub trait ChatRepository {
    async fn read_messages(&self, channel_id: String, after: usize, limit: usize) -> Result<Vec<Message>, InternalError>; 
    async fn post_message(&self, cmd: CreateMessageCommand) -> Result<Message, InternalError>; 
    async fn edit_message(&self, cmd: UpdateMessageCommand) -> Result<Message, ChatError>;
    async fn delete_message(&self, cmd: DeleteMessageCommand) -> Result<(), ChatError>;
    async fn get_message(&self, message_id: Uuid) -> Result<Message, ChatError>;
}

#[async_trait]
pub trait ChatService {
    async fn channel_exists(&self, channel_id: &str) -> bool;
    async fn is_member(&self, user_id: Uuid, channel_id: &str) -> bool;
    async fn create_channel(&self, channel_id: String);
    async fn delete_channel(&self, channel_id: &str);
    async fn add_member(&self, user_id: Uuid, channel_id: String) -> Result<(), InternalError>;
    async fn remove_member(&self, user_id: Uuid, channel_id: &str) -> Result<(), InternalError>;

    async fn read_messages(&self, user_id: Uuid, channel_id: String, after: usize, limit: usize) -> Result<Vec<Message>, ChatError>;
    async fn post_message(&self, cmd: CreateMessageCommand) -> Result<Message, ChatError>;
    async fn edit_message(&self, cmd: UpdateMessageCommand) -> Result<Message, ChatError>;
    async fn delete_message(&self, cmd: DeleteMessageCommand) -> Result<(), ChatError>;
}

#[async_trait]
pub trait ChatEventBus: Send + Sync {
    fn new_channel(&self, channel_id: String);
    fn close_channel(&self, channel_id: String);
    fn subscribe(&self, channel_id: String) -> Option<broadcast::Receiver<ChatEvent>>;
    fn publish(&self, channel_id: String, event: ChatEvent);
    async fn message_posted(&self, message: Message);
    async fn message_edited(&self, message: Message);
    async fn message_deleted(&self, channel_id: String, id: Uuid);
}
