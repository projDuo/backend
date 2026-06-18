use crate::domain::DateTimeWithTimeZone;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GameHistory {
    pub id: Uuid,
    pub account_id: Uuid,
    pub game_id: Uuid,
    pub placement: u32,
    pub points: u64,
    pub cards_had: u64,
    pub participants: Vec<Uuid>,
    pub finished_at: DateTimeWithTimeZone,
}

impl GameHistory {
    pub fn new(
        id: Uuid,
        account_id: Uuid,
        game_id: Uuid,
        placement: u32,
        points: u64,
        cards_had: u64,
        participants: Vec<Uuid>,
        finished_at: DateTimeWithTimeZone,
    ) -> Self {
        Self {
            id,
            account_id,
            game_id,
            placement,
            points,
            cards_had,
            participants,
            finished_at,
        }
    }

    pub fn create(
        account_id: Uuid,
        game_id: Uuid,
        placement: u32,
        points: u64,
        cards_had: u64,
        participants: Vec<Uuid>,
    ) -> Self {
        GameHistory::new(
            uuid::Uuid::new_v4(),
            account_id,
            game_id,
            placement,
            points,
            cards_had as u64,
            participants.to_vec(),
            chrono::Utc::now().into(),
        )
    }
}
