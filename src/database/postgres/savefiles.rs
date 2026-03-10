use super::entities::*;
use crate::{database::core::RepositoryError, domain};
use crate::service::{accounts::Service, payloads::accounts as payloads};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use savefiles::Model;

pub type Savefiles = super::Repository<domain::savefiles::Savefile, prelude::Savefiles>;

impl From<Model> for domain::savefiles::Savefile {
    fn from(value: Model) -> Self {
        Self { 
            id: value.id,
            games_played: value.games_played,
            points: value.points,
            cards_had: value.cards_had,
            wins: value.wins,
            loses: value.loses,
            max_points: value.max_points
        }
    }
}