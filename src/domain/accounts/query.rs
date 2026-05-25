use uuid::Uuid;
use super::DateTimeWithTimeZone;

#[derive(Debug, Clone)]
pub struct AccountPublic {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}