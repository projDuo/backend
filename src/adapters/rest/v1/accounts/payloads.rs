use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::{
    DateTimeWithTimeZone,
    accounts::Account,
};

#[derive(Debug, Serialize)]
pub struct AccountReadPublic {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<Account> for AccountReadPublic {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            login: value.login.to_string(),
            display_name: value.display_name.map(|v| v.to_string()),
            created_at: value.created_at
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Register { //структура, яка задає які поля запит на регістрацію повинен містити
    pub login: String, //логін
    pub password: String, //пароль
}