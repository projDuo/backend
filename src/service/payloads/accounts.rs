use uuid::Uuid;
use crate::domain::DateTimeWithTimeZone;
use crate::domain::accounts::Account;

pub struct AccountCreate {
    pub login: String,
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl AccountCreate {
    pub fn new(login: String, password: String, display_name: Option<String>, created_at: DateTimeWithTimeZone) -> Self {
        Self {
            login,
            password,
            display_name,
            created_at,
        }
    }
}

impl From<AccountCreate> for Account {
    fn from(value: AccountCreate) -> Self {
        Self {
            id: Uuid::now_v7(),
            login: value.login,
            password: value.password,
            display_name: value.display_name,
            created_at: value.created_at,
        }
    }
}

pub struct AccountUpdate {
    pub id: Uuid,
    pub login: String,
    pub password: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}