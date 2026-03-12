use getset::Getters;
use uuid::Uuid;

use crate::domain::{DateTimeWithTimeZone, sessions::SessionError};
use super::value_objects::*;

#[derive(Getters)]
pub struct Session {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub")]
    account_id: Uuid,
    #[getset(get = "pub")]
    created_at: DateTimeWithTimeZone,
    #[getset(get = "pub")]
    pub expires_at: ExpiresAt,
    #[getset(get = "pub")]
    pub is_revoked: bool,
    #[getset(get = "pub")]
    pub token: String,
}

impl Session {
    pub fn new(
        id: Uuid,
        account_id: Uuid,
        created_at: DateTimeWithTimeZone,
        expires_at: ExpiresAt,
        is_revoked: bool,
        token: String,
    ) -> Self {
        Self {
            id,
            account_id,
            created_at,
            expires_at,
            is_revoked,
            token,
        }
    }

    pub fn validate(&self) -> Result<(), SessionError> {
        let now = chrono::Utc::now();
        
        if !(now >= self.created_at) { return Err(SessionError::NotYetValid) };
        if !(now < self.expires_at.as_utc()) { return Err(SessionError::Expired) };
        if self.is_revoked { return Err(SessionError::Revoked) };

        Ok(())
    }
}