use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;
use super::errors::*;
use super::commands::*;
use super::InternalError;

#[async_trait]
pub trait SavefilesRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Savefile>, InternalError>; 
    async fn insert_savefile(&self, cmd: InitSavefileRequest) -> Result<Savefile, SavefileError>;
    async fn update_savefile(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError>;
    async fn delete_savefile(&self, id: Uuid) -> Result<(), SavefileError>;
}

#[async_trait]
pub trait SavefilesService {
    async fn init(&self, id: Uuid) -> Result<Savefile, SavefileError>;
    async fn load(&self, id: Uuid) -> Result<Savefile, SavefileError>;
    async fn save(&self, cmd: UpdateSavefileRequest) -> Result<Savefile, SavefileError>;
    async fn delete(&self, id: Uuid) -> Result<(), SavefileError>;
}