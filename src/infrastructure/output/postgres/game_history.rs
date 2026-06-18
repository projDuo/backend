use async_trait::async_trait;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde_json::from_str;

use crate::domain::game_history::*;
use super::InternalError;
use super::entities::game_histories::*;

impl From<Model> for GameHistory {
    fn from(value: Model) -> Self {
        let participants = from_str::<Vec<uuid::Uuid>>(&value.participants)
            .unwrap_or_default();

        Self {
            id: value.id,
            account_id: value.account_id,
            game_id: value.game_id,
            placement: value.placement as u32,
            points: value.points as u64,
            cards_had: value.cards_had as u64,
            participants,
            finished_at: value.finished_at,
        }
    }
}

impl From<DbErr> for GameHistoryError {
    fn from(value: DbErr) -> Self {
        if let Some(sql_err) = value.sql_err() {
            if matches!(sql_err, sea_orm::SqlErr::UniqueConstraintViolation(_)) {
                return GameHistoryError::Duplicate;
            }
        }

        Self::Infrastructure(value.into())
    }
}

#[async_trait]
impl GameHistoryRepository for super::Postgres {
    async fn insert_history(&self, cmd: GameHistory) -> Result<GameHistory, GameHistoryError> {
        let active_model = ActiveModel {
            id: Set(cmd.id),
            account_id: Set(cmd.account_id),
            game_id: Set(cmd.game_id),
            placement: Set(cmd.placement as i32),
            points: Set(cmd.points as i64),
            cards_had: Set(cmd.cards_had as i64),
            participants: Set(serde_json::to_string(&cmd.participants).unwrap_or_default()),
            finished_at: Set(cmd.finished_at),
            ..Default::default()
        };

        Entity::insert(active_model)
            .exec(&self.db)
            .await?;

        let entity = Entity::find_by_id(cmd.id)
            .one(&self.db)
            .await?
            .expect("Failed to retrieve a game history record immediately after creation.");

        Ok(entity.into())
    }

    async fn list_by_account(&self, account_id: uuid::Uuid, after: Option<uuid::Uuid>, limit: Option<u32>) -> Result<Vec<GameHistory>, InternalError> {
        let mut query = Entity::find()
            .filter(Column::AccountId.eq(account_id))
            .order_by_desc(Column::FinishedAt);

        if let Some(after_id) = after {
            if let Some(after_row) = Entity::find_by_id(after_id).one(&self.db).await? {
                query = query.filter(Column::FinishedAt.lt(after_row.finished_at));
            }
        }

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let rows = query.all(&self.db).await?;

        Ok(rows.into_iter().map(GameHistory::from).collect())
    }
}
