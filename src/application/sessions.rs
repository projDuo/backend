
use crate::domain::sessions::*;
use async_trait::async_trait;
use uuid::Uuid;
pub struct Service<R: SessionsRepository> {
    repo: R,
}

impl<R> Service<R>
where R: SessionsRepository {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> SessionsService for Service<R>
where R: SessionsRepository + Send + Sync {
    async fn id_by_token(&self, token: HashedToken) -> Result<Uuid, SessionError> {
        self.repo.id_by_token(token).await?
            .ok_or(SessionError::Invalid)
    }

    async fn create(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError> {
        self.repo.insert_session(cmd).await
    }

    async fn update(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError> {
        self.repo.update_session(cmd).await
    }
}
