use serde::{ Serialize, Deserialize };
use structural_convert::StructuralConvert;
use uuid::Uuid;
use crate::domain::game::{query, entities};

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(query::PlayerPrivateQuery))]
pub struct PlayerPrivateQuery {
    pub id: Uuid,
    pub cards: u8,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(entities::Direction))]
pub enum Direction { //Список напрямків ходів
    Next,
    Previous,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(entities::Element))]
pub enum Element {
    Water,
    Fire,
    Wood,
    Earth,
    Air,
    Energy,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(entities::Effect))]
pub enum Effect {
    Atk(u8),
    Flow,
    Stun,
    Add(u8),
}

#[derive(Debug, Clone, Serialize)]
pub struct Card { //Структура, що описує карту
    pub element: Element,
    pub effect: Effect, 
}
impl From<entities::Card> for Card {
    fn from(value: entities::Card) -> Self {
        Self { element: (*value.element()).into(), effect: (*value.effect()).into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Turn {
    pub player: Option<Uuid>,
    pub card: Card,
}
impl From<entities::Turn> for Turn {
    fn from(value: entities::Turn) -> Self {
        Self { player: *value.player(), card: (*value.card()).into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResult { //Гравець, що вибув з гри
    id: Uuid, //Ідентифікатор
    cards_had: usize, //Кількість карт які гравець мав за гру загалом
}
impl From<entities::PlayerResult> for PlayerResult {
    fn from(value: entities::PlayerResult) -> Self {
        Self { id: *value.id(), cards_had: *value.cards_had() }
    }
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(query::GameQuery))]
pub struct GameQuery {
    pub id: Uuid,
    pub card: Card,
    pub history: Vec<Turn>,
    pub turn: usize,
    pub direction: Direction,
    pub players: Vec<Uuid>,
    pub players_active: Vec<PlayerPrivateQuery>,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(query::GameNewTurnQuery))]
pub struct GameNewTurnQuery {
    pub id: Uuid,
    pub card: Card,
    pub turn: usize,
    pub direction: Direction,
    pub players_active: Vec<PlayerPrivateQuery>,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(query::GameOverQuery))]
pub struct GameOverQuery {
    pub id: Uuid,
    pub card: Card,
    pub turn: usize,
    pub direction: Direction,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Play { //структура, яка описує необхідні значення в шляху
    pub id: Uuid, //ідентифікатор кімнати
    pub card_id: Option<usize>, //індекс карти у векторі карт гравця
}

#[derive(Debug, Clone, StructuralConvert, Serialize)]
#[convert(from(query::PlayCard))]
pub enum PlayCard {
    NextTurn(GameNewTurnQuery),
    GameOver(GameOverQuery)
}