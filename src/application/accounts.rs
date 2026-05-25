
use crate::domain::accounts::*;
use async_trait::async_trait;
use secrecy::SecretString;
use uuid::Uuid;
use crate::domain::InternalError;

#[derive(Clone)]
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
    async fn read_account(&self, id: Uuid) -> Result<Account, AccountError> {
        self.repo.find_by_id(id).await?
            .ok_or(AccountError::NotFound)
    }

    async fn read_account_by_login(&self, login: String) -> Result<Account, AccountError> {
        self.repo.find_by_login(login).await?
            .ok_or(AccountError::NotFound)
    }

    async fn read_account_by_id_or_login(&self, id: &str) -> Result<Account, AccountError> {
        if let Ok(v) = uuid::Uuid::try_parse(id) {
            self.read_account(v).await
        } else {
            self.read_account_by_login(id.to_string()).await
        }
    }

    async fn register(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError> {
        self.repo.insert_account(cmd).await
    }

    async fn login(&self, login: String, password: SecretString) -> Result<Account, AccountError> {
        let account = self.repo.find_by_login(login).await?
            .ok_or(AccountError::InvalidCredentials)?;

        if !account.password.verify(password).await { 
            return Err(AccountError::InvalidCredentials);
        };

        Ok(account)
    }
    
    async fn is_login_taken(&self, login: String) -> Result<bool, InternalError> {
        let account = self.repo.find_by_login(login).await?;
        Ok(account.is_some())
    }
}