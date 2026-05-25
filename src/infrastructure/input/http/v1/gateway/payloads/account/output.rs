use serde::Serialize;
use uuid::Uuid;
use crate::domain::{
    self, 
    DateTimeWithTimeZone,
    accounts::query as domain_payloads
};
use structural_convert::StructuralConvert;

#[derive(Debug, Serialize, Clone, StructuralConvert)]
#[convert(from(domain_payloads::AccountPublic))]
pub struct AccountReadPublic {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<domain::accounts::Account> for AccountReadPublic {
    fn from(value: domain::accounts::Account) -> Self {
        Self {
            id: value.id,
            login: value.login.to_string(),
            display_name: value.display_name.map(|v| v.to_string()),
            created_at: value.created_at
        }
    }
}