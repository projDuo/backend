use uuid::Uuid;

use crate::domain::DateTimeWithTimeZone;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: String,
    pub author: Uuid,
    pub content: String,
    pub posted_at: DateTimeWithTimeZone,
    pub edited_at: Option<DateTimeWithTimeZone>,
}

impl Message {
    pub fn new(
        id: Uuid,
        channel_id: String,
        author: Uuid,
        content: String,
        posted_at: DateTimeWithTimeZone,
        edited_at: Option<DateTimeWithTimeZone>,
    ) -> Self {
        Self { id, channel_id, author, content, posted_at, edited_at }
    }
}