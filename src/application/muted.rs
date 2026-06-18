use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::muted::*;

pub struct Service<R: MutedRepository + ?Sized> {
    repo: Arc<R>,
}

impl<R> Service<R>
where
    R: MutedRepository + ?Sized,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> MutedService for Service<R>
where
    R: MutedRepository + Send + Sync + ?Sized,
{
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), MutedError> {
        if blocker_id == blocked_id {
            return Err(MutedError::CannotMuteSelf);
        }

        if self.repo.is_user_muted(blocker_id, blocked_id).await? {
            return Err(MutedError::AlreadyMuted);
        }

        self.repo.block_user(blocker_id, blocked_id).await?;
        Ok(())
    }

    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), MutedError> {
        if blocker_id == blocked_id {
            return Err(MutedError::CannotMuteSelf);
        }

        if !self.repo.is_user_muted(blocker_id, blocked_id).await? {
            return Err(MutedError::NotMuted);
        }

        self.repo.unblock_user(blocker_id, blocked_id).await?;
        Ok(())
    }

    async fn is_user_muted(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, MutedError> {
        self.repo.is_user_muted(blocker_id, blocked_id).await.map_err(MutedError::from)
    }

    async fn get_muted_users(&self, blocker_id: Uuid) -> Result<Vec<MutedEntry>, MutedError> {
        self.repo.get_muted_users(blocker_id).await.map_err(MutedError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::InternalError;
    use tokio::sync::Mutex;
    use std::collections::HashSet;
    use std::sync::Arc;

    struct DummyRepo { set: Mutex<HashSet<(Uuid, Uuid)>> }
    impl DummyRepo { fn new() -> Self { Self { set: Mutex::new(HashSet::new()) } } }

    #[async_trait]
    impl MutedRepository for DummyRepo {
        async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError> {
            self.set.lock().await.insert((blocker_id, blocked_id));
            Ok(())
        }
        async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), InternalError> {
            self.set.lock().await.remove(&(blocker_id, blocked_id));
            Ok(())
        }
        async fn is_user_muted(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, InternalError> {
            Ok(self.set.lock().await.contains(&(blocker_id, blocked_id)))
        }
        async fn get_muted_users(&self, blocker_id: Uuid) -> Result<Vec<MutedEntry>, InternalError> {
            Ok(self.set.lock().await.iter().filter(|(b, _)| *b == blocker_id).map(|(_, blocked)| MutedEntry::new(*blocked, chrono::Utc::now().into())).collect())
        }
    }

    #[tokio::test]
    async fn block_unblock() {
        let repo = Arc::new(DummyRepo::new());
        let svc = Service::new(repo.clone());
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();

        svc.block_user(a, b).await.unwrap();
        assert!(svc.is_user_muted(a, b).await.unwrap());
        svc.unblock_user(a, b).await.unwrap();
        assert!(!svc.is_user_muted(a, b).await.unwrap());
    }
}
