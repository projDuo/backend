use uuid::Uuid;

use crate::domain::sessions::HashedToken;

#[derive(Debug, Clone)]
pub struct TokenData {
    pub account_id: Uuid,
    pub session_id: Uuid,
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: HashedToken,
}