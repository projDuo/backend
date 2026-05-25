use crate::domain::game::GameError;

use super::*;
impl Game {
    pub fn new(
        id: Uuid,
        history: Vec<Turn>,
        turn: usize,
        direction: Direction,
        players: Vec<Uuid>,
    ) -> Self {
        Self { id, history, turn, direction, players }
    }

    pub fn init(players: Vec<Uuid>) -> Self {
        Self { 
            id: Uuid::now_v7(),
            history: vec!(
                Turn::new(
                    None,
                    Card::new(Element::Energy, Effect::Flow)
                )
            ), 
            turn: 0,
            direction: Direction::Next,
            players
        }
    }

    pub fn get_player_index(&self, player_id: Uuid) -> Result<usize, GameError> { //метод для знаходження індексу гравця у векторі за його id
        self.players.iter().position(|v| *v == player_id)
            .ok_or(GameError::PlayerNotFound) //видати помилку якщо не знайдено
    }
}