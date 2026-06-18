use serde::{Deserialize, Serialize};

use crate::domain::auth::TokenPair;

#[derive(Debug, Deserialize)]
pub struct Login { //структура, яка задає які поля запит на авторизацію повинен містити
    pub login: String, //логін
    pub password: String, //пароль
}

#[derive(Debug, Deserialize)]
pub struct Refresh {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPairPayload {
    pub access_token: String,
    pub refresh_token: String,
}

impl From<TokenPair> for TokenPairPayload {
    fn from(value: TokenPair) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token.to_string()
        }
    }
}