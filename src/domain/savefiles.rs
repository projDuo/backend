use uuid::Uuid;
use thiserror::Error;

use crate::domain::{ErrorType, Identifiable};

pub struct Savefile {
    pub id: Uuid,
    pub games_played: i64,
    pub points: i64,
    pub cards_had: i64,
    pub wins: i32,
    pub loses: i32,
    pub max_points: i16,
}

impl Identifiable for Savefile {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Error)]
pub enum Error {

}

impl ErrorType for Savefile {
    type Error = Error;
}