use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait RolesRepository {
    async fn is_mod(&self, id: Uuid) -> bool;
}

#[async_trait]
pub trait RolesService {
    async fn is_mod(&self, id: Uuid) -> bool;
}