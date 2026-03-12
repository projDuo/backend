use uuid::Uuid;

use crate::domain::DateTimeWithTimeZone;
use super::value_objects::*;

pub struct CreateSessionRequest {
    pub account_id: Uuid,
}

impl CreateSessionRequest {
    pub fn new(account_id: Uuid) -> Self {
        Self { account_id }
    }
}

pub struct UpdateSessionRequest {
    pub id: Uuid,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub is_revoked: Option<bool>,
}

pub struct UpdateByTokenSessionRequest {
    pub id: SessionToken,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub is_revoked: Option<bool>,
}
