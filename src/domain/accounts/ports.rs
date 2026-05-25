use async_trait::async_trait;
use secrecy::SecretString;
use uuid::Uuid;

use crate::domain::accounts::AccountPublic;

use super::entities::*;
use super::errors::*;
use super::commands::*;
use super::InternalError;

#[async_trait]
pub trait AccountsRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, InternalError>; 
    async fn find_by_login(&self, login: String) -> Result<Option<Account>, InternalError>; 
    async fn insert_account(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError>;
    async fn update_account(&self, cmd: UpdateAccountRequest) -> Result<Account, AccountError>;
    async fn delete_account(&self, id: Uuid) -> Result<(), AccountError>;
    async fn find_many_accounts_by_id_public(&self, ids: Vec<Uuid>) -> Result<Vec<AccountPublic>, InternalError>;
}

#[async_trait]
pub trait AccountsService {
    async fn read_account(&self, id: Uuid) -> Result<Account, AccountError>;
    async fn read_account_by_login(&self, login: String) -> Result<Account, AccountError>;
    async fn read_account_by_id_or_login(&self, id: &str) -> Result<Account, AccountError>;
    async fn register(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError>;
    async fn login(&self, login: String, password: SecretString) -> Result<Account, AccountError>;
    async fn is_login_taken(&self, login: String) -> Result<bool, InternalError>;
}