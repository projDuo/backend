use crate::domain::{game_history::{GameHistory, GameHistoryError, GameHistoryRepository, GameHistoryService}, InternalError};
use async_trait::async_trait;
use uuid::Uuid;

pub struct Service<R: GameHistoryRepository> {
    repo: R,
}

impl<R> Service<R>
where
    R: GameHistoryRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> GameHistoryService for Service<R>
where
    R: GameHistoryRepository + Send + Sync,
{
    async fn record_history(&self, cmd: GameHistory) -> Result<GameHistory, GameHistoryError> {
        self.repo.insert_history(cmd).await
    }

    async fn list_player_history(&self, account_id: Uuid, after: Option<Uuid>, limit: Option<u32>) -> Result<Vec<GameHistory>, InternalError> {
        self.repo.list_by_account(account_id, after, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct DummyRepo {
        history: Mutex<Vec<GameHistory>>,
    }

    impl DummyRepo {
        fn new() -> Self {
            Self {
                history: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl GameHistoryRepository for DummyRepo {
        async fn insert_history(&self, cmd: GameHistory) -> Result<GameHistory, GameHistoryError> {
            let mut history = self.history.lock().await;
            history.push(cmd.clone());
            Ok(cmd)
        }

        async fn list_by_account(&self, account_id: Uuid, _after: Option<Uuid>, _limit: Option<u32>) -> Result<Vec<GameHistory>, InternalError> {
            let history = self.history.lock().await;
            Ok(history.iter().filter(|entry| entry.account_id == account_id).cloned().collect())
        }
    }

    #[tokio::test]
    async fn record_and_list_history() {
        let repo = DummyRepo::new();
        let service = Service::new(repo);
        let account_id = Uuid::now_v7();
        let game_id = Uuid::now_v7();

        let record = GameHistory::create(account_id, game_id, 1, 42, 5, vec![account_id]);
        let saved = service.record_history(record.clone()).await.expect("insert history");
        assert_eq!(saved.account_id, account_id);
        assert_eq!(saved.points, 42);

        let listed = service.list_player_history(account_id, None, None).await.expect("list history");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, saved.id);
    }
}
