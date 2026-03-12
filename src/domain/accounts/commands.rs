use super::value_objects::*;

use uuid::Uuid;

pub struct CreateAccountRequest {
    pub login: Login,
    pub password: Password,
}

pub struct UpdateAccountRequest {
    pub id: Uuid,
    pub login: Option<Login>,
    pub password: Option<Password>,
    pub display_name: Option<DisplayName>,
}