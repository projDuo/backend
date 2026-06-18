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
            HashedToken::new(jwt.refresh_token.clone()).to_string(),
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
        
        session.token = HashedToken::new(jwt.refresh_token.clone()).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::InternalError;
    use async_trait::async_trait;
    use secrecy::SecretString;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct DummyAccountsService {
        id: Uuid,
        login: String,
        password: Password,
    }

    #[async_trait]
    impl AccountsService for DummyAccountsService {
        async fn read_account(&self, _id: Uuid) -> Result<Account, AccountError> { unimplemented!() }
        async fn read_account_by_login(&self, _login: String) -> Result<Account, AccountError> { unimplemented!() }
        async fn read_account_by_id_or_login(&self, _id: &str) -> Result<Account, AccountError> { unimplemented!() }
        async fn update_account(&self, _cmd: UpdateAccountRequest) -> Result<Account, AccountError> { unimplemented!() }
        async fn register(&self, _cmd: CreateAccountRequest) -> Result<Account, AccountError> { unimplemented!() }
        async fn login(&self, login: String, _password: SecretString) -> Result<Account, AccountError> {
            if login == self.login {
                Ok(Account::new(
                    self.id,
                    Login::from_raw(self.login.clone()),
                    self.password.clone(),
                    None,
                    chrono::Utc::now().into(),
                ))
            } else {
                Err(AccountError::NotFound)
            }
        }
        async fn is_login_taken(&self, _login: String) -> Result<bool, InternalError> { Ok(false) }
    }

    struct DummyTokenProvider;
    impl TokenProvider for DummyTokenProvider {
        fn refresh_expires_after(&self) -> chrono::Duration { chrono::Duration::minutes(10) }
        fn access_expires_after(&self) -> chrono::Duration { chrono::Duration::minutes(5) }
        fn generate_pair(&self, account_id: Uuid, session_id: Uuid) -> Result<TokenPair, AuthError> {
            Ok(TokenPair { access_token: format!("access-{}", session_id), refresh_token: format!("refresh-{}", account_id) })
        }
        fn generate_token(&self, _account_id: Uuid, _session_id: Uuid) -> Result<String, AuthError> {
            Ok("access-token".to_string())
        }
        fn verify_token(&self, token: SecretString) -> Result<TokenData, AuthError> {
            let value = token.expose_secret();
            if let Some(rest) = value.strip_prefix("access-") {
                let session_id = Uuid::parse_str(rest).map_err(|_| AuthError::Session(SessionError::Invalid))?;
                return Ok(TokenData::new(Uuid::nil(), session_id));
            }
            Err(AuthError::Session(SessionError::Invalid))
        }
    }

    struct DummySessionsRepo {
        sessions: Mutex<HashMap<Uuid, Session>>,
    }

    #[async_trait]
    impl SessionsRepository for DummySessionsRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, InternalError> {
            let sessions = self.sessions.lock().await;
            Ok(sessions.get(&id).map(|session| Session::new(*session.id(), *session.account_id(), session.created_at().clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), session.is_revoked, session.token.clone())))
        }

        async fn find_by_token(&self, token: HashedToken) -> Result<Option<Session>, InternalError> {
            let sessions = self.sessions.lock().await;
            Ok(sessions.values().find(|session| session.token == token.to_string()).map(|session| Session::new(*session.id(), *session.account_id(), session.created_at().clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), session.is_revoked, session.token.clone())))
        }

        async fn id_by_token(&self, token: HashedToken) -> Result<Option<Uuid>, InternalError> {
            let sessions = self.sessions.lock().await;
            Ok(sessions.values().find(|session| session.token == token.to_string()).map(|session| *session.id()))
        }

        async fn insert_session(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError> {
            let session = Session::new(cmd.id, cmd.account_id, chrono::Utc::now().into(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), false, cmd.token.clone());
            self.sessions.lock().await.insert(cmd.id, Session::new(cmd.id, cmd.account_id, session.created_at().clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), false, cmd.token.clone()));
            Ok(session)
        }

        async fn update_session(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError> {
            let expires_at = cmd.expires_at.unwrap_or_else(|| chrono::Utc::now().into());
            let token = cmd.token.unwrap_or_default();
            let session = Session::new(cmd.id, Uuid::nil(), chrono::Utc::now().into(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), cmd.is_revoked.unwrap_or(false), token.clone());
            self.sessions.lock().await.insert(cmd.id, Session::new(cmd.id, Uuid::nil(), session.created_at().clone(), crate::domain::sessions::value_objects::ExpiresAt::new(chrono::Duration::minutes(10)), session.is_revoked, token.clone()));
            Ok(session)
        }

        async fn revoke_session(&self, id: Uuid) -> Result<bool, InternalError> {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(&id) {
                session.is_revoked = true;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn revoke_all_sessions(&self, account_id: Uuid) -> Result<bool, InternalError> {
            let mut sessions = self.sessions.lock().await;
            let mut found = false;
            for session in sessions.values_mut() {
                if *session.account_id() == account_id {
                    session.is_revoked = true;
                    found = true;
                }
            }
            Ok(found)
        }
    }

    #[tokio::test]
    async fn create_and_verify_token() {
        let account_id = Uuid::now_v7();
        let login = "user123".to_string();
        let password = Password::from_hash("Password1".to_string());
        let accounts_service = Arc::new(DummyAccountsService { id: account_id, login: login.clone(), password });
        let sessions_repo = DummySessionsRepo { sessions: Mutex::new(HashMap::new()) };
        let token_provider = DummyTokenProvider;
        let auth_service = Service::new(accounts_service, sessions_repo, token_provider);

        let jwt = auth_service.create(login.clone(), SecretString::new("Password1".to_string().into_boxed_str())).await.expect("create auth");
        let token_data = auth_service.verify(SecretString::new(jwt.access_token.clone().into_boxed_str())).await.expect("verify token");
        assert_eq!(token_data.session_id.to_string(), jwt.access_token.trim_start_matches("access-").to_string());
    }
}
