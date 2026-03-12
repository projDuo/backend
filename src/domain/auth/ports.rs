use async_trait::async_trait;

use crate::domain::accounts::*;
use crate::domain::sessions::*;
use super::errors::*;

#[async_trait]
pub trait AuthService {
    async fn create(&self, login: &Login, password: &Password) -> Result<Session, AuthError>;
    async fn revoke(&self, token: &SessionToken) -> Result<bool, AuthError>;
    async fn revoke_all(&self, token: &SessionToken) -> Result<bool, AuthError>;
}