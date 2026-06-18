use uuid::Uuid;
use serde::Serialize;
use structural_convert::StructuralConvert;

use crate::domain::chat::events::ChatEvent;

pub mod output;

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[serde(tag = "event", content = "data")]
#[convert(from(ChatEvent))]
pub enum ChatEventOut {
    MessagePosted(output::MessageResponse),
    MessageEdited(output::MessageResponse),
    MessageDeleted(Uuid),
}
