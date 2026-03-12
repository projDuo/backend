use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;
use super::value_objects::*;
use super::errors::*;
use super::commands::*;
use super::InternalRepositoryError;

#[async_trait]
pub trait AccountsRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, InternalRepositoryError>; 
    async fn find_by_login(&self, login: &Login) -> Result<Option<Account>, InternalRepositoryError>; 
    async fn insert_account(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError>;
    async fn update_account(&self, cmd: UpdateAccountRequest) -> Result<Account, AccountError>;
    async fn delete_account(&self, id: Uuid) -> Result<(), AccountError>;
}

#[async_trait]
pub trait AccountsService {
    async fn register(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError>;
    async fn login(&self, login: &Login, password: &Password) -> Result<Account, AccountError>;
    async fn is_login_taken(&self, login: &Login) -> Result<bool, InternalRepositoryError>;
}