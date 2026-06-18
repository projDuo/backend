use std::collections::HashSet;
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{chat::*, InternalError};
use crate::domain::muted::{MutedEntry, MutedRepository};
use crate::infrastructure::output::memory::channel::ChannelStorage;

#[derive(Clone)]
pub struct Chat {
    chat_repo: Arc<dyn ChatRepository + Send + Sync>,
    chat_event_bus: Arc<dyn ChatEventBus + Send + Sync>,
    channels: Arc<ChannelStorage>,
    muted_repo: Arc<dyn MutedRepository + Send + Sync>,
}

impl Chat {
    pub fn new(
        chat_repo: Arc<dyn ChatRepository + Send + Sync>,
        chat_event_bus: Arc<dyn ChatEventBus + Send + Sync>,
        channels: Arc<ChannelStorage>,
        muted_repo: Arc<dyn MutedRepository + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            chat_repo,
            chat_event_bus,
            channels,
            muted_repo,
        })
    }

    async fn ensure_channel_access(&self, user_id: Uuid, channel_id: &str) -> Result<(), ChatError> {
        if !self.channels.channel_exists(channel_id).await {
            return Err(ChatError::ChannelNotFound);
        }
        if !self.channels.is_member(user_id, channel_id).await {
            return Err(ChatError::Unauthorized);
        }
        Ok(())
    }
}

#[async_trait]
impl ChatService for Chat {
    async fn channel_exists(&self, channel_id: &str) -> bool {
        self.channels.channel_exists(channel_id).await
    }

    async fn is_member(&self, user_id: Uuid, channel_id: &str) -> bool {
        self.channels.is_member(user_id, channel_id).await
    }

    async fn create_channel(&self, channel_id: String) {
        self.channels.create_channel(channel_id).await;
    }

    async fn delete_channel(&self, channel_id: &str) {
        self.channels.delete_channel(channel_id).await;
    }

    async fn add_member(&self, user_id: Uuid, channel_id: String) -> Result<(), InternalError> {
        self.channels.add_member(user_id, channel_id).await
    }

    async fn remove_member(&self, user_id: Uuid, channel_id: &str) -> Result<(), InternalError> {
        self.channels.remove_member(user_id, channel_id).await
    }
    async fn read_messages(&self, user_id: Uuid, channel_id: String, after: usize, limit: usize) -> Result<Vec<Message>, ChatError> {
        self.ensure_channel_access(user_id, &channel_id).await?;

        let muted_users = self
            .muted_repo
            .get_muted_users(user_id)
            .await
            .map_err(|_| ChatError::Internal)?;

        let muted_ids: HashSet<Uuid> = muted_users
            .into_iter()
            .map(|entry| entry.blocked_id)
            .collect();

        let messages = self.chat_repo.read_messages(channel_id, after, limit).await
            .map_err(|_| ChatError::Internal)?;

        Ok(messages
            .into_iter()
            .filter(|message| !muted_ids.contains(&message.author))
            .collect())
    }

    async fn post_message(&self, cmd: CreateMessageCommand) -> Result<Message, ChatError> {
        self.ensure_channel_access(cmd.author, &cmd.channel_id).await?;
        let message = self.chat_repo.post_message(cmd).await
            .map_err(|_| ChatError::Internal)?;

        self.chat_event_bus.message_posted(message.clone()).await;
        Ok(message)
    }

    async fn edit_message(&self, cmd: UpdateMessageCommand) -> Result<Message, ChatError> {
        let message = self.chat_repo.get_message(cmd.message_id).await?;
        self.ensure_channel_access(cmd.author, &message.channel_id).await?;
        let message = self.chat_repo.edit_message(cmd).await?;
        self.chat_event_bus.message_edited(message.clone()).await;
        Ok(message)
    }

    async fn delete_message(&self, cmd: DeleteMessageCommand) -> Result<(), ChatError> {
        let message = self.chat_repo.get_message(cmd.message_id).await?;
        self.ensure_channel_access(cmd.author, &message.channel_id).await?;
        self.chat_repo.delete_message(cmd).await?;
        self.chat_event_bus.message_deleted(message.channel_id, message.id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use uuid::Uuid;

    struct DummyChatRepository {
        blocked_user: Uuid,
    }

    impl DummyChatRepository {
        fn new(blocked_user: Uuid) -> Self {
            Self { blocked_user }
        }
    }

    #[async_trait]
    impl ChatRepository for DummyChatRepository {
        async fn read_messages(&self, _channel_id: String, _after: usize, _limit: usize) -> Result<Vec<Message>, InternalError> {
            let user_one = Uuid::now_v7();
            Ok(vec![
                Message::new(Uuid::now_v7(), "room".to_string(), user_one, "hello".to_string(), chrono::Utc::now().into(), None),
                Message::new(Uuid::now_v7(), "room".to_string(), self.blocked_user, "blocked".to_string(), chrono::Utc::now().into(), None),
            ])
        }
        async fn post_message(&self, _cmd: CreateMessageCommand) -> Result<Message, InternalError> { unimplemented!() }
        async fn edit_message(&self, _cmd: UpdateMessageCommand) -> Result<Message, ChatError> { unimplemented!() }
        async fn delete_message(&self, _cmd: DeleteMessageCommand) -> Result<(), ChatError> { unimplemented!() }
        async fn get_message(&self, _message_id: Uuid) -> Result<Message, ChatError> { unimplemented!() }
    }

    struct DummyChatEventBus;
    #[async_trait]
    impl ChatEventBus for DummyChatEventBus {
        fn new_channel(&self, _channel_id: String) {}
        fn close_channel(&self, _channel_id: String) {}
        fn subscribe(&self, _channel_id: String) -> Option<broadcast::Receiver<ChatEvent>> { None }
        fn publish(&self, _channel_id: String, _event: ChatEvent) {}
        async fn message_posted(&self, _message: Message) {}
        async fn message_edited(&self, _message: Message) {}
        async fn message_deleted(&self, _channel_id: String, _id: Uuid) {}
    }

    struct DummyMutedRepo {
        blocked_id: Uuid,
    }

    impl DummyMutedRepo {
        fn new(blocked_id: Uuid) -> Self {
            Self { blocked_id }
        }
    }

    #[async_trait]
    impl MutedRepository for DummyMutedRepo {
        async fn block_user(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> Result<(), InternalError> { Ok(()) }
        async fn unblock_user(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> Result<(), InternalError> { Ok(()) }
        async fn is_user_muted(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> Result<bool, InternalError> { Ok(false) }
        async fn get_muted_users(&self, _blocker_id: Uuid) -> Result<Vec<MutedEntry>, InternalError> {
            Ok(vec![MutedEntry::new(self.blocked_id, chrono::Utc::now().into())])
        }
    }

    #[tokio::test]
    async fn read_messages_filters_blocked_author() {
        let channel_id = "room".to_string();
        let user_id = Uuid::now_v7();
        let blocked_author = Uuid::now_v7();
        let channels = ChannelStorage::new().into_arc();
        channels.create_channel(channel_id.clone()).await;
        channels.add_member(user_id, channel_id.clone()).await.expect("add member");
        let chat = Chat::new(
            Arc::new(DummyChatRepository::new(blocked_author)),
            Arc::new(DummyChatEventBus),
            channels,
            Arc::new(DummyMutedRepo::new(blocked_author)),
        );

        let messages = chat.read_messages(user_id, channel_id.clone(), 0, 10).await.expect("read messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].channel_id, channel_id);
    }
}
