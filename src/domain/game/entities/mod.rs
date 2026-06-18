use getset::Getters;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum Direction { //Список напрямків ходів
    Next,
    Previous,
}
pub mod direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
    Water,
    Fire,
    Wood,
    Earth,
    Air,
    Energy,
}
pub mod element;

#[derive(Debug, Clone, Copy)]
pub enum Effect {
    Atk(u8),
    Flow,
    Stun,
    Add(u8),
}
pub mod effect;

#[derive(Debug, Clone, Copy, Getters)]
pub struct Card { //Структура, що описує карту
    #[getset(get = "pub")]
    element: Element,
    #[getset(get = "pub")]
    effect: Effect, 
}
pub mod card;

#[derive(Debug, Clone, Copy, Getters)]
pub struct Turn {
    #[getset(get = "pub")]
    player: Option<Uuid>,
    #[getset(get = "pub")]
    card: Card,
}
pub mod turn;

#[derive(Debug, Clone, Getters)]
pub struct Game { //Структура, що описує гру
    #[getset(get = "pub")]
    id: Uuid,
    pub(crate) history: Vec<Turn>, //Попередня зіграна карта
    pub(crate) turn: usize, //індекс гравця, що ходить
    pub(crate) turn_enforced_at: i64, //unix timestamp ms when turn is auto-forced
    pub(crate) turn_timeout_secs: u64, //max seconds allowed for a turn
    pub(crate) direction: Direction, //напрямок ходів
    #[getset(get = "pub")]
    players: Vec<Uuid>
}
pub mod game;

#[derive(Debug, Clone, Getters)]
pub struct Player { //Структура, яка описує гравця гри
    #[getset(get = "pub")]
    id: Uuid, //Ідентифікатор
    #[getset(get = "pub")]
    cards: Vec<Card>, //Вектор карт
    cards_count: usize, //Список карт які гравець тримав загалом
}
pub mod player;

#[derive(Debug, Clone, Getters)]
pub struct PlayerResult { //Гравець, що вибув з гри
    #[getset(get = "pub")]
    id: Uuid, //Ідентифікатор
    #[getset(get = "pub")]
    cards_had: usize, //Кількість карт які гравець мав за гру загалом
}
pub mod player_result;