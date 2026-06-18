use uuid::Uuid;
use serde::Serialize;
use crate::domain::chat::entities::Message;

#[derive(Debug, Serialize, Clone)]
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
            edited_at: value.edited_at.map(|dt| dt.to_rfc3339()),
        }
    }
}
