use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;
use super::value_objects::*;
use super::errors::*;
use super::commands::*;
use super::InternalRepositoryError;

#[async_trait]
pub trait SessionsRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, InternalRepositoryError>; 
    async fn find_by_token(&self, token: &SessionToken) -> Result<Option<Session>, InternalRepositoryError>;
    async fn id_by_token(&self, token: &SessionToken) -> Result<Option<Uuid>, InternalRepositoryError>;
    async fn insert_session(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError>;
    async fn update_session(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError>;
    async fn revoke_session(&self, token: &SessionToken) -> Result<bool, InternalRepositoryError>;
    async fn revoke_all_sessions(&self, account_id: Uuid) -> Result<bool, InternalRepositoryError>;
}

#[async_trait]
pub trait SessionsService {
    async fn id_by_token(&self, token: &SessionToken) -> Result<Uuid, SessionError>;
    async fn create(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError>;
    async fn update(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError>;
}