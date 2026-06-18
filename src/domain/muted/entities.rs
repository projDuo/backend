use crate::domain::DateTimeWithTimeZone;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MutedEntry {
    pub blocked_id: Uuid,
    pub blocked_at: DateTimeWithTimeZone,
}

impl MutedEntry {
    pub fn new(blocked_id: Uuid, blocked_at: DateTimeWithTimeZone) -> Self {
        Self { blocked_id, blocked_at }
    }
}
