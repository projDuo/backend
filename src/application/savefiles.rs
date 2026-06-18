
use crate::domain::savefiles::*;
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::InternalError;

pub struct Service<R: SavefilesRepository> {
    repo: R,
}

impl<R> Service<R>
where R: SavefilesRepository {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> SavefilesService for Service<R>
where R: SavefilesRepository + Send + Sync {
    async fn init(&self, id: Uuid) -> Result<Savefile, SavefileError> {
        let cmd = InitSavefileRequest { id };
        self.repo.insert_savefile(cmd).await
    }
    async fn load(&self, id: Uuid) -> Result<Savefile, SavefileError> {
        let savefile = if let Some(v) = self.repo.find_by_id(id).await? {
            v
        } else {
            self.init(id).await?
        };

        Ok(savefile)
    }
    async fn save(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError> {
        self.repo.update_savefile(cmd).await
    }
    async fn delete(&self, id: Uuid) -> Result<(), SavefileError> {
        self.repo.delete_savefile(id).await
    }

    async fn get_the_best(&self) -> Result<Vec<Savefile>, InternalError> {
        self.repo.get_the_best().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;
    use std::collections::HashMap;

    struct DummyRepo { inner: Mutex<HashMap<Uuid, Savefile>> }
    impl DummyRepo { fn new() -> Self { Self { inner: Mutex::new(HashMap::new()) } } }

    #[async_trait]
    impl SavefilesRepository for DummyRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Savefile>, InternalError> {
            Ok(self.inner.lock().await.get(&id).cloned())
        }
        async fn insert_savefile(&self, cmd: InitSavefileRequest) -> Result<Savefile, SavefileError> {
            let save = Savefile::new(cmd.id, 0, 0, 0, 0, 0, 0);
            self.inner.lock().await.insert(cmd.id, save.clone());
            Ok(save)
        }
        async fn update_savefile(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError> {
            let mut inner = self.inner.lock().await;
            let sf = inner.get_mut(&cmd.id).ok_or(SavefileError::NotFound)?;
            if let Some(g) = cmd.games_played { sf.games_played = g; }
            Ok(sf.clone())
        }
        async fn delete_savefile(&self, _id: Uuid) -> Result<(), SavefileError> { Ok(()) }
        async fn get_the_best(&self) -> Result<Vec<Savefile>, InternalError> { Ok(self.inner.lock().await.values().cloned().collect()) }
    }

    #[tokio::test]
    async fn init_and_load() {
        let repo = DummyRepo::new();
        let svc = Service::new(repo);
        let id = Uuid::now_v7();
        let s = svc.init(id).await.expect("init");
        assert_eq!(*s.id(), id);
        let loaded = svc.load(id).await.expect("load");
        assert_eq!(*loaded.id(), id);
    }
}