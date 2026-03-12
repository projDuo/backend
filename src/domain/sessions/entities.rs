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
    expires_at: DateTimeWithTimeZone,
    #[getset(get = "pub")]
    is_revoked: bool,
    #[getset(get = "pub")]
    token: SessionToken,
}

impl Session {
    pub fn new(
        id: Uuid,
        account_id: Uuid,
        created_at: DateTimeWithTimeZone,
        expires_at: DateTimeWithTimeZone,
        is_revoked: bool,
        token: SessionToken,
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
        if !(now < self.expires_at) { return Err(SessionError::Expired) };
        if self.is_revoked { return Err(SessionError::Revoked) };

        Ok(())
    }
}