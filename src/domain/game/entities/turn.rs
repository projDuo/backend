use super::*;
impl Turn {
    pub fn new(player: Option<Uuid>, card: Card) -> Self {
        Self { player, card }
    }
}