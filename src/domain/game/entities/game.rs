use crate::domain::game::GameError;

use super::*;
impl Game {
    pub fn new(
        id: Uuid,
        history: Vec<Turn>,
        turn: usize,
        turn_enforced_at: i64,
        turn_timeout_secs: u64,
        direction: Direction,
        players: Vec<Uuid>,
    ) -> Self {
        Self { id, history, turn, turn_enforced_at, turn_timeout_secs, direction, players }
    }

    pub fn init(players: Vec<Uuid>, turn_timeout_secs: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let turn_enforced_at = now + (turn_timeout_secs as i64 * 1000);
        Self { 
            id: Uuid::now_v7(),
            history: vec!(
                Turn::new(
                    None,
                    Card::new(Element::Energy, Effect::Flow)
                )
            ), 
            turn: 0,
            turn_enforced_at,
            turn_timeout_secs,
            direction: Direction::Next,
            players
        }
    }

    pub fn get_player_index(&self, player_id: Uuid) -> Result<usize, GameError> { //метод для знаходження індексу гравця у векторі за його id
        self.players.iter().position(|v| *v == player_id)
            .ok_or(GameError::PlayerNotFound) //видати помилку якщо не знайдено
    }
}