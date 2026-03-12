
use crate::domain::accounts::*;
use async_trait::async_trait;
use secrecy::SecretString;
pub struct Service<R: AccountsRepository> {
    repo: R,
}

impl<R> Service<R>
where R: AccountsRepository {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> AccountsService for Service<R>
where R: AccountsRepository + Send + Sync {
    async fn register(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError> {
        self.repo.insert_account(cmd).await
    }

    async fn login(&self, login: String, password: SecretString) -> Result<Account, AccountError> {
        let login = Login::from_raw(login);
        let account = self.repo.find_by_login(login).await?
            .ok_or(AccountError::InvalidCredentials)?;

        if !account.password.verify(password).await { 
            return Err(AccountError::InvalidCredentials);
        };

        Ok(account)
    }
    
    async fn is_login_taken(&self, login: Login) -> Result<bool, InternalError> {
        let account = self.repo.find_by_login(login).await?;
        Ok(account.is_some())
    }
}