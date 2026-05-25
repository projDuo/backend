use sea_orm::{
    prelude::DateTimeWithTimeZone,
    DerivePartialModel
};
use structural_convert::StructuralConvert;
use super::prelude::Accounts;
use uuid::Uuid;
use crate::domain::accounts::query;

#[derive(DerivePartialModel, StructuralConvert)]
#[sea_orm(entity = "Accounts")]
#[convert(into(query::AccountPublic))]
pub struct AccountPublic {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}