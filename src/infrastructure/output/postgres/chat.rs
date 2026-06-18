use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DbErr, EntityTrait, QuerySelect, QueryFilter, ColumnTrait, Order, QueryOrder, Set};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::chat::*;
use super::InternalError;
use super::entities::chat_messages::*;

impl From<Model> for Message {
    fn from(value: Model) -> Self {
        Self::new(
            value.id,
            value.channel_id,
            value.author_id,
            value.content,
            value.posted_at.into(),
            value.edited_at.map(|dt| dt.into()),
        )
    }
}

impl From<DbErr> for ChatError {
    fn from(value: DbErr) -> Self {
        tracing::error!("Database error: {:?}", value);
        ChatError::ChannelNotFound
    }
}

#[async_trait]
impl ChatRepository for super::Postgres {
    async fn read_messages(&self, channel_id: String, after: usize, limit: usize) -> Result<Vec<Message>, InternalError> {
        let models = Entity::find()
            .filter(Column::ChannelId.eq(channel_id))
            .order_by(Column::PostedAt, Order::Desc)
            .offset(after as u64)
            .limit(limit as u64)
            .all(&self.db)
            .await?;

        let messages: Vec<Message> = models
            .into_iter()
            .map(Message::from)
            .collect();

        Ok(messages)
    }

    async fn post_message(&self, cmd: CreateMessageCommand) -> Result<Message, InternalError> {
        let active_model = ActiveModel {
            id: Set(Uuid::new_v4()),
            channel_id: Set(cmd.channel_id),
            author_id: Set(cmd.author),
            content: Set(cmd.content),
            posted_at: Set(Utc::now().into()),
            edited_at: Set(None),
        };

        let model = Entity::insert(active_model)
            .exec(&self.db)
            .await?;

        let message = self.get_message(model.last_insert_id).await
            .map_err(|_| InternalError("Failed to retrieve message after creation".to_string()))?;

        Ok(message)
    }

    async fn edit_message(&self, cmd: UpdateMessageCommand) -> Result<Message, ChatError> {
        let message = self.get_message(cmd.message_id).await?;

        // Check authorization
        if message.author != cmd.author {
            return Err(ChatError::Unauthorized);
        }

        let mut active_model = ActiveModel::new();
        active_model.set(Column::Id, cmd.message_id.into());
        active_model.set(Column::Content, cmd.content.into());
        // `edited_at` is nullable in the DB, so SeaORM expects `Option<DateTime<FixedOffset>>`.
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        active_model.set(
            Column::EditedAt,
            sea_orm::Value::ChronoDateTimeWithTimeZone(Some(now)),
        );

        let updated = Entity::update(active_model).exec(&self.db).await?;
        Ok(updated.into())
    }

    async fn delete_message(&self, cmd: DeleteMessageCommand) -> Result<(), ChatError> {
        let message = self.get_message(cmd.message_id).await?;

        // Check authorization
        if message.author != cmd.author {
            return Err(ChatError::Unauthorized);
        }

        Entity::delete_by_id(cmd.message_id).exec(&self.db).await?;
        Ok(())
    }

    async fn get_message(&self, message_id: Uuid) -> Result<Message, ChatError> {
        let message = Entity::find_by_id(message_id)
            .one(&self.db)
            .await?
            .ok_or(ChatError::MessageNotFound)?;

        Ok(message.into())
    }
}
