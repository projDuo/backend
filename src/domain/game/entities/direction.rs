use super::Direction;
impl Direction {
    pub fn switch(&mut self) -> &mut Self { //перемикач напрямку
        *self = match self {
            Direction::Next => Direction::Previous,
            Direction::Previous => Direction::Next,
        };
        self
    }
}