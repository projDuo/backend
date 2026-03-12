use sea_orm::{
    ActiveModelBehavior,
    ActiveModelTrait,
    ColumnTrait,
    DbErr,
    EntityTrait,
    QueryFilter,
    QuerySelect,
    Set,
    prelude::Expr,
};
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::sessions::*;
use super::InternalRepositoryError;
use super::entities::sessions::*;

impl From<Model> for Session {
    fn from(value: Model) -> Self {
        Self::new(
            value.id,
            value.account_id,
            value.created_at,
            value.expires_at,
            value.is_revoked,
            value.token.into()
        )
    }
}

impl From<DbErr> for SessionError {
    fn from(value: DbErr) -> Self {
        if let Some(sql_err) = value.sql_err() {
            if matches!(sql_err, sea_orm::SqlErr::UniqueConstraintViolation(_)) {
                return SessionError::Duplicate;
            }
        }

        Self::Infrastructure(value.into())
    }
}

#[async_trait]
impl SessionsRepository for super::Postgres {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Session>, InternalRepositoryError> {
        let entity = Entity::find_by_id(id).one(&self.db).await?
            .map(Session::from);
        
        Ok(entity)
    }

    async fn find_by_token(&self, token: &SessionToken) -> Result<Option<Session>, InternalRepositoryError> {
        let entity = Entity::find()
            .filter(Column::Token.eq(token.to_string())).one(&self.db).await?
            .map(Session::from);
        
        Ok(entity)
    }

    async fn id_by_token(&self, token: &SessionToken) -> Result<Option<Uuid>, InternalRepositoryError> {
        let entity = Entity::find()
            .filter(Column::Token.eq(token.to_string()))
            .select_only()
            .column(Column::Id)
            .into_tuple::<Uuid>()
            .one(&self.db).await?;
        
        Ok(entity)
    }

    async fn insert_session(&self, cmd: CreateSessionRequest) -> Result<Session, SessionError> {
        let active_model = ActiveModel {
            account_id: Set(cmd.account_id),
            ..Default::default()
        };

        let insert_result = Entity::insert(active_model)
            .exec(&self.db)
            .await?;

        let id = insert_result.last_insert_id;
    
        let entity = self.find_by_id(id)
            .await?
            .expect("Failed to retrieve a session right after creating it.");
    
        Ok(entity)
    }

    async fn update_session(&self, cmd: UpdateSessionRequest) -> Result<Session, SessionError> {
        let mut active_model = ActiveModel::new();
        active_model.set(Column::Id, cmd.id.into());

        if let Some(v) = cmd.expires_at {
            active_model.set(Column::ExpiresAt, v.into());
        }

        if let Some(v) = cmd.is_revoked {
            active_model.set(Column::IsRevoked, v.into());
        }

        let model = Entity::update(active_model).exec(&self.db).await?;
            Ok(model.into())
        }

        async fn revoke_session(&self, token: &SessionToken) -> Result<bool, InternalRepositoryError> {
            let result = Entity::update_many()
                .col_expr(Column::IsRevoked, Expr::value(true))
                .filter(Column::Token.eq(token.as_str()))
                .exec(&self.db)
                .await
                .map_err(InternalRepositoryError::from)?;
            
            Ok(result.rows_affected > 0)
        }

        async fn revoke_all_sessions(&self, account_id: Uuid) -> Result<bool, InternalRepositoryError> {
            let result = Entity::update_many()
                .col_expr(Column::IsRevoked, Expr::value(true))
                .filter(Column::AccountId.eq(account_id))
                .exec(&self.db)
                .await
                .map_err(InternalRepositoryError::from)?;
            
            Ok(result.rows_affected > 0)
        }

}
