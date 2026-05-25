use std::hash::Hash;
use super::*;
impl PlayerResult {
    pub fn new(
        id: Uuid,
        cards_had: usize,
    ) -> Self {
        Self { id, cards_had }
    }
}

impl Hash for PlayerResult { //Налаштування хешування Player, його унікальність повинна визначатися тільки за id
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for PlayerResult { } //Порівняння за id
impl PartialEq for PlayerResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<Player> for PlayerResult {
    fn from(value: Player) -> Self {
        Self {
            id: *value.id(),
            cards_had: value.cards_count,
        }
    }
}