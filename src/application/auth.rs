use crate::domain::accounts::*;
use crate::domain::auth::*;
use crate::domain::sessions::*;

use async_trait::async_trait;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Service<A, S, T>
where
    A: AccountsService,
    S: SessionsRepository,
    T: TokenProvider,
{
    accounts_service: Arc<A>,
    sessions_repo: S,
    token_provider: T,
}

impl<A, S, T> Service<A, S, T>
where
    A: AccountsService,
    S: SessionsRepository,
    T: TokenProvider,
{
    pub fn new(
        accounts_service: Arc<A>,
        sessions_repo: S,
        token_provider: T,
    ) -> Self {
        Self { accounts_service, sessions_repo, token_provider }
    }
}

#[async_trait]
impl<A, S, T> AuthService for Service<A, S, T> 
where
    A: AccountsService + Send + Sync,
    S: SessionsRepository + Send + Sync,
    T: TokenProvider + Send + Sync,
{
    async fn create(&self, login: String, password: SecretString) -> Result<TokenPair, AuthError> {
        let account = self.accounts_service.login(login, password).await?;
        let session_id = uuid::Uuid::now_v7();

        let jwt = self.token_provider.generate_pair(account.id, session_id)?;
        
        let create_session_req = CreateSessionRequest::new(
            session_id,
            account.id,
            jwt.refresh_token.to_string(),
            ExpiresAt::new(self.token_provider.refresh_expires_after()).into(),
        );
        self.sessions_repo.insert_session(create_session_req).await?;
    
        Ok(jwt)
    }

    async fn refresh(&self, refresh_token: SecretString) -> Result<TokenPair, AuthError> {
        let refresh_token = HashedToken::new(refresh_token.expose_secret().to_string());

        let mut session = self.sessions_repo.find_by_token(refresh_token).await?
            .ok_or(SessionError::Invalid)?;

        session.validate()?;

        let jwt = self.token_provider.generate_pair(*session.account_id(), *session.id())?;
        
        session.token = jwt.refresh_token.to_string();
        session.expires_at = ExpiresAt::new(self.token_provider.refresh_expires_after());

        self.sessions_repo.update_session(session.into()).await?;

        Ok(jwt)
    }

    async fn revoke(&self, token_data: TokenData) -> Result<bool, AuthError> {
        let result = self.sessions_repo.revoke_session(token_data.session_id).await?;
        Ok(result)
    }

    async fn revoke_all(&self, token_data: TokenData) -> Result<bool, AuthError> {
        let result = self.sessions_repo.revoke_all_sessions(token_data.account_id).await?;
        Ok(result)
    }

    async fn verify(&self, token: SecretString) -> Result<TokenData, AuthError> {
        self.token_provider.verify_token(token)
    }
}