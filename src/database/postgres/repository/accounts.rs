use super::entities::*;
use crate::domain;
use crate::payloads::accounts as payloads;
use sea_orm::Set;

impl From<payloads::AccountCreate> for accounts::ActiveModel {
    fn from(value: payloads::AccountCreate) -> Self {
        Self {
            login: Set(value.login),
            password: Set(value.password),
            display_name: Set(value.display_name),
            created_at: Set(value.created_at),
            ..Default::default()
        }
    }
}

impl From<payloads::AccountUpdate> for accounts::ActiveModel {
    fn from(value: payloads::AccountUpdate) -> Self {
        Self {
            id: Set(value.id),
            login: Set(value.login),
            password: Set(value.password),
            display_name: Set(value.display_name),
            created_at: Set(value.created_at),
            ..Default::default()
        }
    }
}

impl From<accounts::Model> for domain::accounts::Account {
    fn from(value: accounts::Model) -> Self {
        Self {
            id: value.id,
            login: value.login,
            password: value.password,
            display_name: value.display_name,
            created_at: value.created_at,
        }
    }
}