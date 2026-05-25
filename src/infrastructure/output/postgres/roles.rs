use std::str::FromStr;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::Postgres;
use async_trait::async_trait;
use crate::domain::roles::*;
use super::entities::roles_assigned::*;

#[async_trait]
impl RolesRepository for Postgres {
    async fn is_mod(&self, id: Uuid) -> bool {
        let mod_role = Uuid::from_str("019d1667-3712-7be7-a3ac-ad9d1917b195").expect("invalid mod role");

        if let Ok(Some(_)) = Entity::find()
            .filter(Column::AccountId.eq(id))
            .filter(Column::RoleId.eq(mod_role))
            .one(&self.db)
            .await {
                true
            } else {false}
    }
} 