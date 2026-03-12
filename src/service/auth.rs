use crate::domain::accounts::*;
use crate::domain::auth::*;
use crate::domain::sessions::*;

use async_trait::async_trait;

struct Service<A, S>
where
    A: AccountsService,
    S: SessionsRepository,
{
    accounts_service: A,
    sessions_repo: S,
}

#[async_trait]
impl<A, S> AuthService for Service<A, S> 
where
    A: AccountsService + Send + Sync,
    S: SessionsRepository + Send + Sync
{
    async fn create(&self, login: &Login, password: &Password) -> Result<Session, AuthError> {
        let account = self.accounts_service.login(login, password).await?;
        
        let create_session_req = CreateSessionRequest::new(account.id);
        let session = self.sessions_repo.insert_session(create_session_req).await?;
    
        Ok(session)
    }
    async fn revoke(&self, token: &SessionToken) -> Result<bool, AuthError> {
        let session = self.sessions_repo.find_by_token(token).await?
            .ok_or(SessionError::Invalid)?;

        session.validate()?;
        let result = self.sessions_repo.revoke_session(token).await?;
        
        Ok(result)
    }

    async fn revoke_all(&self, token: &SessionToken) -> Result<bool, AuthError> {
        let session = self.sessions_repo.find_by_token(token).await?
            .ok_or(SessionError::Invalid)?;
        
        let result = self.sessions_repo.revoke_all_sessions(*session.account_id()).await?;
        Ok(result)
    }
}