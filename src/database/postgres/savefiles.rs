use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DbErr, EntityTrait, Set};
use async_trait::async_trait;

use crate::domain::savefiles::*;
use super::InternalRepositoryError;
use super::entities::savefiles::*;

impl From<Model> for Savefile {
    fn from(value: Model) -> Self {
        Self::new(
            value.id,
            value.games_played as u64,
            value.points as u64,
            value.cards_had as u64,
            value.wins as u32,
            value.loses as u32,
            value.max_points as u16
        )
    }
}

impl From<DbErr> for SavefileError {
    fn from(value: DbErr) -> Self {
        if let Some(sql_err) = value.sql_err() {
            if matches!(sql_err, sea_orm::SqlErr::UniqueConstraintViolation(_)) {
                return SavefileError::Duplicate;
            }
        }

        Self::Infrastructure(value.into())
    }
}

#[async_trait]
impl SavefilesRepository for super::Postgres {
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Savefile>, InternalRepositoryError> {
        let entity = Entity::find_by_id(id).one(&self.db).await?
            .map(Savefile::from);
        
        Ok(entity)
    }

    async fn insert_savefile(&self, cmd: InitSavefileRequest) -> Result<Savefile, SavefileError> {
        let active_model = ActiveModel {
            id: Set(cmd.id),
            ..Default::default()
        };

        let insert_result = Entity::insert(active_model)
            .exec(&self.db)
            .await?;

        let id = insert_result.last_insert_id;
    
        let entity = self.find_by_id(id)
            .await?
            .expect("Failed to retrieve a savefile right after creating it.");
    
        Ok(entity)
    }

    async fn update_savefile(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError> {
        let mut active_model = ActiveModel::new();
        active_model.set(Column::Id, cmd.id.into());

        if let Some(v) = cmd.games_played {
            active_model.set(Column::GamesPlayed, (v as i64).into());
        }

        if let Some(v) = cmd.points {
            active_model.set(Column::Points, (v as i64).into());
        }

        if let Some(v) = cmd.cards_had {
            active_model.set(Column::CardsHad, (v as i64).into());
        }

        if let Some(v) = cmd.wins {
            active_model.set(Column::Wins, (v as i32).into());
        }

        if let Some(v) = cmd.loses {
            active_model.set(Column::Loses, (v as i32).into());
        }

        let model = Entity::update(active_model).exec(&self.db).await?;
        Ok(model.into())
    }

    async fn delete_savefile(&self, id: uuid::Uuid) -> Result<(), SavefileError> {
        Entity::delete_by_id(id).exec(&self.db).await?;
        
        Ok(())
    }
}
