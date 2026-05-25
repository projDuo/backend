use crate::domain::roles::*;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct Service<R: RolesRepository> {
    repo: R,
}

impl<R> Service<R>
where R: RolesRepository {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R> RolesService for Service<R>
where R: RolesRepository + Send + Sync {
    async fn is_mod(&self, id: Uuid) -> bool {
        self.repo.is_mod(id).await
    }
}