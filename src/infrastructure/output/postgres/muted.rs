use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, DeleteResult};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::muted::*;
use crate::domain::InternalError;
use super::entities::muted::*;

#[async_trait]
impl MutedRepository for super::Postgres {
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError> {
        let existing = Entity::find()
            .filter(Column::BlockerId.eq(blocker_id))
            .filter(Column::BlockedId.eq(blocked_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(());
        }

        let active_model = ActiveModel {
            id: Set(Uuid::new_v4()),
            blocker_id: Set(blocker_id),
            blocked_id: Set(blocked_id),
            blocked_at: Set(Utc::now().into()),
        };

        active_model.insert(&self.db).await?;
        Ok(())
    }

    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError> {
        let DeleteResult { rows_affected, .. } = Entity::delete_many()
            .filter(Column::BlockerId.eq(blocker_id))
            .filter(Column::BlockedId.eq(blocked_id))
            .exec(&self.db)
            .await?;

        if rows_affected == 0 {
            return Ok(());
        }

        Ok(())
    }

    async fn is_user_muted(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, InternalError> {
        let entry = Entity::find()
            .filter(Column::BlockerId.eq(blocker_id))
            .filter(Column::BlockedId.eq(blocked_id))
            .one(&self.db)
            .await?;
        Ok(entry.is_some())
    }

    async fn get_muted_users(&self, blocker_id: Uuid) -> Result<Vec<MutedEntry>, InternalError> {
        let entries = Entity::find()
            .filter(Column::BlockerId.eq(blocker_id))
            .all(&self.db)
            .await?;

        Ok(entries
            .into_iter()
            .map(|model| MutedEntry::new(model.blocked_id, model.blocked_at))
            .collect())
    }
}
