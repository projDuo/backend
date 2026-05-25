use serde::Serialize;
use crate::{ domain::{ DateTimeWithTimeZone, activity } };
use uuid::Uuid;
#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub id: Uuid,
    pub room: Option<String>,
    pub game: Option<Uuid>,
    pub updated_at: DateTimeWithTimeZone,
}
impl From<activity::entities::Activity> for Activity {
    fn from(value: activity::entities::Activity) -> Self {
        Self { id: *value.id(), room: value.room, game: value.game, updated_at: value.updated_at }
    }
}