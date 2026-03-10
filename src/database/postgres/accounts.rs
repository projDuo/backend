use super::entities::*;
use crate::{database::core::RepositoryError, domain};
use crate::service::{accounts::Service, payloads::accounts as payloads};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

pub type Accounts = super::Repository<domain::accounts::Account, prelude::Accounts>;

impl Service for Accounts {
    type Error = RepositoryError;

    async fn read_by_login(&self, login: &str) -> Result<domain::accounts::Account, crate::database::core::RepositoryError> {
        let model = prelude::Accounts::find()
            .filter(accounts::Column::Login.eq(login))
            .one(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        if let Some(v) = model {
            Ok(v.into())
        } else {
            Err(RepositoryError::NotFound("E".to_string()))
        }
    }
}

impl From<payloads::AccountCreate> for accounts::ActiveModel {
    fn from(value: payloads::AccountCreate) -> Self {
        Self {
            login: Set(value.login),
            password: Set(value.password),
            display_name: Set(value.display_name),
            created_at: Set(value.created_at),
            ..Default::default()
        }
    }
}

impl From<payloads::AccountUpdate> for accounts::ActiveModel {
    fn from(value: payloads::AccountUpdate) -> Self {
        Self {
            id: Set(value.id),
            login: Set(value.login),
            password: Set(value.password),
            display_name: Set(value.display_name),
            created_at: Set(value.created_at),
            ..Default::default()
        }
    }
}

impl From<accounts::Model> for domain::accounts::Account {
    fn from(value: accounts::Model) -> Self {
        Self {
            id: value.id,
            login: value.login,
            password: value.password,
            display_name: value.display_name,
            created_at: value.created_at,
        }
    }
}