use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::domain::InternalError;

pub struct ChannelStorage {
    channels: DashMap<String, HashSet<Uuid>>,
}

impl ChannelStorage {
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
        }
    }
}

impl ChannelStorage {
    pub async fn channel_exists(&self, channel_id: &str) -> bool {
        self.channels.contains_key(channel_id)
    }

    pub async fn is_member(&self, user_id: Uuid, channel_id: &str) -> bool {
        self.channels
            .get(channel_id)
            .is_some_and(|members| members.contains(&user_id))
    }

    pub async fn create_channel(&self, channel_id: String) {
        self.channels.entry(channel_id).or_insert_with(HashSet::new);
    }

    pub async fn delete_channel(&self, channel_id: &str) {
        self.channels.remove(channel_id);
    }

    pub async fn add_member(&self, user_id: Uuid, channel_id: String) -> Result<(), InternalError> {
        let mut members = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| InternalError(format!("Channel {channel_id} not found")))?;
        members.insert(user_id);
        Ok(())
    }

    pub async fn remove_member(&self, user_id: Uuid, channel_id: &str) -> Result<(), InternalError> {
        if let Some(mut members) = self.channels.get_mut(channel_id) {
            members.remove(&user_id);
        }
        Ok(())
    }
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}
