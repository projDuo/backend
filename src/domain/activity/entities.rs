use chrono::Utc;
use crate::domain::DateTimeWithTimeZone;
use getset::Getters;
use uuid::Uuid;


#[derive(Debug, Clone, Getters)]
pub struct Activity {
    #[getset(get = "pub")]
    id: Uuid,
    pub room: Option<String>,
    pub game: Option<Uuid>,
    pub(crate) updated_at: DateTimeWithTimeZone,
}
impl Activity {
    pub fn new(
        id: Uuid,
        room: Option<String>,
        game: Option<Uuid>,
    ) -> Self {
        Self { id, room, game, updated_at: Utc::now().into() }
    }
}