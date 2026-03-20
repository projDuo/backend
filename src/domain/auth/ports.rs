use async_trait::async_trait;
use chrono::Duration;
use secrecy::SecretString;
use uuid::Uuid;

use super::errors::*;
use super::entities::*;

#[async_trait]
pub trait AuthService {
    async fn create(&self, login: String, password: SecretString) -> Result<TokenPair, AuthError>;
    async fn refresh(&self, refresh_token: SecretString) -> Result<TokenPair, AuthError>;
    async fn revoke(&self, token_data: TokenData) -> Result<bool, AuthError>;
    async fn revoke_all(&self, token_data: TokenData) -> Result<bool, AuthError>;
    async fn verify(&self, token_data: SecretString) -> Result<TokenData, AuthError>;
}

pub trait TokenProvider: Send + Sync {
    fn refresh_expires_after(&self) -> Duration;
    fn access_expires_after(&self) -> Duration;
    fn generate_pair(&self, account_id: Uuid, session_id: Uuid) -> Result<TokenPair, AuthError>;
    fn generate_token(&self, account_id: Uuid, session_id: Uuid) -> Result<String, AuthError>;
    fn verify_token(&self, token: SecretString) -> Result<TokenData, AuthError>;
}

