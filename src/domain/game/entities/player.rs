use uuid::Uuid;
use super::*;

impl Player {
    pub fn init(id: Uuid) -> Self {
        let mut cards: Vec<Card> = Vec::new();
        for _i in 0..8 { //додавання восьми карт
            cards.push(rand::random());
        }
        let cards_count = cards.len();
        
        Self { 
            id,
            cards,
            cards_count
        }
    }

    pub fn add_card(&mut self, card: Card) { //Метод додавання карти
        self.cards.push(card);
        self.cards_count += 1;
    }

    pub fn remove_card(&mut self, index: usize) -> Card { //Метод видалення карти
        self.cards.remove(index)
    }

    pub fn get_card(&self, index: usize) -> Option<&Card> { //Метод взяття карти
        self.cards.get(index)
    }
}

impl From<Uuid> for Player {
    fn from(value: Uuid) -> Self {
        Self::init(value)
    }
}