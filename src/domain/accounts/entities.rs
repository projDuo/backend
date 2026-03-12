use uuid::Uuid;
use getset::Getters;
use super::value_objects::*;
use crate::domain::DateTimeWithTimeZone;

#[derive(Getters)]
pub struct Account {
    #[getset(get = "pub")]
    pub id: Uuid,
    #[getset(get = "pub")]
    pub login: Login,
    #[getset(get = "pub")]
    pub password: Password,
    #[getset(get = "pub")]
    pub display_name: Option<DisplayName>,
    #[getset(get = "pub")]
    pub created_at: DateTimeWithTimeZone,
}

impl Account {
    pub fn new(
        id: Uuid,
        login: Login,
        password: Password,
        display_name: Option<DisplayName>,
        created_at: DateTimeWithTimeZone,
    ) -> Self {
        Self {id, login, password, display_name, created_at}
    }
}