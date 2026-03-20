
use crate::domain::savefiles::*;
use async_trait::async_trait;
use uuid::Uuid;

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
        self.repo.find_by_id(id).await?
            .ok_or(SavefileError::NotFound)
    }
    async fn save(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError> {
        self.repo.update_savefile(cmd).await
    }
    async fn delete(&self, id: Uuid) -> Result<(), SavefileError> {
        self.repo.delete_savefile(id).await
    }
}