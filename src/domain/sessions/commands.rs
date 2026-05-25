use uuid::Uuid;

use crate::domain::DateTimeWithTimeZone;
use super::entities::*;

pub struct CreateSessionRequest {
    pub id: Uuid,
    pub account_id: Uuid,
    pub token: String,
    pub expires_at: DateTimeWithTimeZone,
}

impl CreateSessionRequest {
    pub fn new(
        id: Uuid,
        account_id: Uuid,
        token: String,
        expires_at: DateTimeWithTimeZone,
    ) -> Self {
        Self { id, account_id, token, expires_at }
    }
}

pub struct UpdateSessionRequest {
    pub id: Uuid,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub is_revoked: Option<bool>,
    pub token: Option<String>,
}

impl From<Session> for UpdateSessionRequest {
    fn from(value: Session) -> Self {
        Self {
            id: *value.id(),
            expires_at: Some(value.expires_at.into()),
            is_revoked: Some(value.is_revoked),
            token: Some(value.token) 
        }
    }
}

