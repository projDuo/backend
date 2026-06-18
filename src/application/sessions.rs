
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::InternalError;
    use tokio::sync::Mutex;
    use std::collections::HashMap;

    struct DummyRepo { inner: Mutex<HashMap<Uuid, (Uuid, crate::domain::DateTimeWithTimeZone, crate::domain::sessions::value_objects::ExpiresAt, bool, String)>> }
    impl DummyRepo { fn new() -> Self { Self { inner: Mutex::new(HashMap::new()) } } }

    #[async_trait]
    impl SessionsRepository for DummyRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, InternalError> {
            let inner = self.inner.lock().await;
            if let Some((account_id, created_at, _expires_at, is_revoked, token)) = inner.get(&id) {
                return Ok(Some(Session::new(id, *account_id, created_at.clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(5)), *is_revoked, token.clone())));
            }
            Ok(None)
        }
        async fn find_by_token(&self, token: HashedToken) -> Result<Option<Session>, InternalError> {
            let inner = self.inner.lock().await;
            for (id, (account_id, created_at, _expires_at, is_revoked, t)) in inner.iter() {
                if t == &token.to_string() {
                    return Ok(Some(Session::new(*id, *account_id, created_at.clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(5)), *is_revoked, t.clone())));
                }
            }
            Ok(None)
        }
        async fn id_by_token(&self, token: HashedToken) -> Result<Option<Uuid>, InternalError> {
            let inner = self.inner.lock().await;
            for (id, (_acc, _created, _exp, _revoked, t)) in inner.iter() {
                if t == &token.to_string() { return Ok(Some(*id)); }
            }
            Ok(None)
        }
        async fn insert_session(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError> {
            let mut inner = self.inner.lock().await;
            inner.insert(cmd.id, (cmd.account_id, cmd.expires_at, crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(5)), false, cmd.token.clone()));
            Ok(Session::new(cmd.id, cmd.account_id, chrono::Utc::now().into(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(5)), false, cmd.token))
        }
        async fn update_session(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError> { Err(SessionError::Invalid) }
        async fn revoke_session(&self, id: Uuid) -> Result<bool, InternalError> { Ok(self.inner.lock().await.get_mut(&id).map(|v| { v.3 = true; true }).unwrap_or(false)) }
        async fn revoke_all_sessions(&self, _account_id: Uuid) -> Result<bool, InternalError> { Ok(true) }
    }

    #[tokio::test]
    async fn create_and_id_by_token() {
        let repo = DummyRepo::new();
        let svc = Service::new(repo);
        let session_id = Uuid::now_v7();
        let account_id = Uuid::now_v7();
        let token = crate::domain::sessions::value_objects::HashedToken::new("refresh".to_string());
        let expires_at = crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(5));
        let session = svc.create(CreateSessionRequest::new(session_id, account_id, token.to_string(), expires_at.into())).await.expect("create");
        let id = svc.id_by_token(token).await.expect("id_by_token");
        assert_eq!(id, *session.id());
    }
}
