use uuid::Uuid;
use std::hash::Hash;

use super::Player;

impl Player {
    pub fn new(id: Uuid, room_id: String, is_ready: bool, points: u64) -> Self {
        Self { id, room_id, is_ready, points }
    }
}

impl PartialEq for Player { //PartialEq для Player, порівняння тільки за полем id
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.room_id == other.room_id
    }
}

impl Hash for Player { //Хешування за полем id
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.room_id.hash(state);
    }
}