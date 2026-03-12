use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use async_trait::async_trait;

use crate::domain::accounts::*;
use super::entities::accounts::*;

impl From<Model> for Account {
    fn from(value: Model) -> Self {
        let display_name = value.display_name.map(|v| {
                    DisplayName::new(v)
        });
        Self::new(value.id, Login::from_raw(value.login), Password::from_hash(value.password), display_name, value.created_at)
    }
}

impl From<DbErr> for AccountError {
    fn from(value: DbErr) -> Self {
        if let Some(sql_err) = value.sql_err() {
            if matches!(sql_err, sea_orm::SqlErr::UniqueConstraintViolation(_)) {
                return AccountError::Duplicate;
            }
        }

        Self::Infrastructure(value.into())
    }
}

#[async_trait]
impl AccountsRepository for super::Postgres {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Account>, InternalRepositoryError> {
        let entity = Entity::find_by_id(id).one(&self.db).await?
            .map(Account::from);
        
        Ok(entity)
    }

    async fn find_by_login(&self, login: &Login) -> Result<Option<Account>, InternalRepositoryError> {
        let entity = Entity::find()
            .filter(Column::Login.eq(login.to_string()))
            .one(&self.db)
            .await?
            .map(Account::from);

        Ok(entity)
    }

    async fn insert_account(&self, cmd: CreateAccountRequest) -> Result<Account, AccountError> {
        let active_model = ActiveModel {
            login: Set(cmd.login.to_string()),
            password: Set(cmd.password.hash()),
            ..Default::default()
        };

        let insert_result = Entity::insert(active_model)
            .exec(&self.db)
            .await?;

        let id = insert_result.last_insert_id;
    
        let entity = self.find_by_id(id)
            .await?
            .expect("Failed to retrieve an account right after creating it.");
    
        Ok(entity)
    }

    async fn update_account(&self, cmd: UpdateAccountRequest) -> Result<Account, AccountError> {
        let mut active_model = ActiveModel::new();
        active_model.set(Column::Id, cmd.id.into());

        if let Some(v) = cmd.login {
            active_model.set(Column::Login, v.to_string().into());
        }

        if let Some(v) = cmd.password {
            active_model.set(Column::Password, v.hash().into());
        }

        if let Some(v) = cmd.display_name {
            active_model.set(Column::DisplayName, v.to_string().into());
        }

        let model = Entity::update(active_model).exec(&self.db).await?;
        Ok(model.into())
    }

    async fn delete_account(&self, id: uuid::Uuid) -> Result<(), AccountError> {
        let result = Entity::delete_by_id(id).exec(&self.db).await?;
        
        if result.rows_affected == 0 { return Err(AccountError::NotFound) };

        Ok(())
    }
}

/*
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
}*/