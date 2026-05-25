
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