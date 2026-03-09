use uuid::Uuid;
use crate::domain::DateTimeWithTimeZone;
use super::Identifiable;

pub struct Account {
    pub id: Uuid,
    pub login: String,
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl Identifiable for Account {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}