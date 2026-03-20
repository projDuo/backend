use uuid::Uuid;

use crate::domain::sessions::HashedToken;

#[derive(Debug, Clone)]
pub struct TokenData {
    pub account_id: Uuid,
    pub session_id: Uuid,
}

impl TokenData {
    pub fn new(account_id: Uuid, session_id: Uuid) -> Self {
        Self {
            account_id,
            session_id
        }
    }
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: HashedToken,
}