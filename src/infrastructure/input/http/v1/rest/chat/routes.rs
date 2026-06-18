use poem::{
    handler,
    web::{Data, Json, Path, Query},
    Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::chat::{ChatService, CreateMessageCommand, DeleteMessageCommand, Message, UpdateMessageCommand};
use crate::AppState;

use super::super::auth::middleware::AuthenticatedUser;

fn default_limit() -> usize {
    100
}

#[derive(Debug, Deserialize)]
pub struct ReadMessagesQuery {
    #[serde(default)]
    pub after: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: String,
    pub author: Uuid,
    pub content: String,
    pub posted_at: String,
    pub edited_at: Option<String>,
}

impl From<Message> for MessageResponse {
    fn from(value: Message) -> Self {
        Self {
            id: value.id,
            channel_id: value.channel_id,
            author: value.author,
            content: value.content,
            posted_at: value.posted_at.to_rfc3339(),
            edited_at: value.edited_at.map(|v| v.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChannelPath {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct MessagePath {
    pub id: String,
    pub message_id: Uuid,
}

#[handler]
pub async fn read_messages(
    Path(ChannelPath { id }): Path<ChannelPath>,
    query: Query<ReadMessagesQuery>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<Vec<MessageResponse>>> {
    let messages = state
        .chats
        .read_messages(user.account_id, id, query.after, query.limit)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(messages))
}

#[handler]
pub async fn post_message(
    Path(ChannelPath { id }): Path<ChannelPath>,
    body: Json<CreateMessageRequest>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<MessageResponse>> {
    let cmd = CreateMessageCommand::new(id, user.account_id, body.content.clone());
    let message = state.chats.post_message(cmd).await?;
    Ok(Json(message.into()))
}

#[handler]
pub async fn edit_message(
    Path(MessagePath { id: _, message_id }): Path<MessagePath>,
    body: Json<UpdateMessageRequest>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<MessageResponse>> {
    let cmd = UpdateMessageCommand::new(message_id, user.account_id, body.content.clone());
    let message = state.chats.edit_message(cmd).await?;
    Ok(Json(message.into()))
}

#[handler]
pub async fn delete_message(
    Path(MessagePath { id: _, message_id }): Path<MessagePath>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<()> {
    let cmd = DeleteMessageCommand::new(message_id, user.account_id);
    state.chats.delete_message(cmd).await?;
    Ok(())
}
