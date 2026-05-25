use super::*;
use uuid::Uuid;
use std::hash::Hash;

impl Room {
    pub fn new(
        id: RoomId,
        name: Name,
        is_public: IsPublic,
        password: Option<PasswordHash>,
        owner: Uuid,
        max_players: MaxPlayers,
    ) -> Self {
        Self { id, name, is_public, password, owner, max_players }
    }
}

impl PartialEq for Room //Власна реалізація ознаки порівняння, таким чином щоб структура Room порівнювалась за полем id
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Room //Реалізація ознаки Hash для Room, таким чином щоб хеш-функція обчислювалась тільки від поля id
 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}