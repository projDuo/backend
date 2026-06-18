use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "muted")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub blocker_id: Uuid,
    pub blocked_id: Uuid,
    pub blocked_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation, Eq, PartialEq)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
