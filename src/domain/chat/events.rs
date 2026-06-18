use uuid::Uuid;

use super::entities::Message;

#[derive(Debug, Clone)]
pub enum ChatEvent {
    MessagePosted(Message),
    MessageEdited(Message),
    MessageDeleted(Uuid),
}