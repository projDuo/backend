use uuid::Uuid;

use crate::domain::savefiles::Savefile;

pub struct InitSavefileRequest {
    pub id: Uuid
}

impl InitSavefileRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

pub struct UpdateSavefileRequest {
    pub id: Uuid,
    pub games_played: Option<u64>,
    pub points: Option<u64>,
    pub cards_had: Option<u64>,
    pub wins: Option<u32>,
    pub loses: Option<u32>,
    pub max_points: Option<u16>,
}

impl From<Savefile> for UpdateSavefileRequest {
    fn from(value: Savefile) -> Self {
        Self {
            id: *value.id(),
            games_played: Some(value.games_played),
            points: Some(value.points),
            cards_had: Some(value.cards_had),
            wins: Some(*value.wins()),
            loses: Some(*value.loses()),
            max_points: Some(*value.max_points())
        }
    }
}