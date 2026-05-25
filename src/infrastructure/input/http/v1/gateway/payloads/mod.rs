pub mod room;
pub mod account;
pub mod game;
pub mod auth;
pub mod activity;

use serde::{Serialize, Deserialize};
use derive_more::From;

use crate::infrastructure::http::v1::rest::errors::ErrorCode;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EventIn {
    Auth(auth::AuthEventIn)
}

#[derive(Debug, Serialize, Clone, From)]
#[serde(untagged)]
pub enum EventOut {
    Error(ErrorCode),
    Auth(auth::AuthEventOut),
    Room(room::RoomEventOut),
    Game(game::GameEventOut),
    Activity(activity::ActivityEventOut),
}
impl EventOut {
    pub fn to_json_string(&self) -> String { //серіалізація у рядок
        serde_json::to_string(self).expect("Failed to serialize Gateway EventOut Payload")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    #[serde(skip_serializing)]
    EventIn(EventIn),
    #[serde(skip_deserializing)]
    EventOut(EventOut),
}