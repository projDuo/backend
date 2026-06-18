use serde::Serialize;
use uuid::Uuid;
use crate::domain::{
    savefiles::Savefile,
};

#[derive(Debug, Serialize)]
pub struct SavefilePublic {
    pub id: Uuid,
    pub games_played: u64,
    pub points: u64,
    pub cards_had: u64,
    pub wins: u32,
    pub loses: u32,
    pub max_points: u64,
}

impl From<Savefile> for SavefilePublic {
    fn from(value: Savefile) -> Self {
        Self {
            id: *value.id(),
            games_played: *value.games_played(),
            points: *value.points(),
            cards_had: *value.cards_had(),
            wins: *value.wins(),
            loses: *value.loses(),
            max_points: *value.max_points(),
        }
    }
}