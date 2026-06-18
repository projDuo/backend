use uuid::Uuid;

pub struct CreateMessageCommand {
    pub channel_id: String,
    pub author: Uuid,
    pub content: String,
}

impl CreateMessageCommand {
    pub fn new(
        channel_id: String,
        author: Uuid,
        content: String,
    ) -> Self {
        Self { channel_id, author, content }
    }
}

pub struct UpdateMessageCommand {
    pub message_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

impl UpdateMessageCommand {
    pub fn new(
        message_id: Uuid,
        author: Uuid,
        content: String,
    ) -> Self {
        Self { message_id, author, content }
    }
}

pub struct DeleteMessageCommand {
    pub message_id: Uuid,
    pub author: Uuid,
}

impl DeleteMessageCommand {
    pub fn new(
        message_id: Uuid,
        author: Uuid,
    ) -> Self {
        Self { message_id, author }
    }
}
