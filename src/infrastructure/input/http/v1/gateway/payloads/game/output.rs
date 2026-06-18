use uuid::Uuid;
use serde::Serialize;
use structural_convert::StructuralConvert;
use crate::domain::game;

#[derive(Debug, Clone, StructuralConvert, Serialize)]
#[convert(from(game::entities::Direction))]
pub enum Direction { //Список напрямків ходів
    Next,
    Previous,
}

#[derive(Debug, Clone, StructuralConvert, Serialize)]
#[convert(from(game::entities::Element))]
pub enum Element {
    Water,
    Fire,
    Wood,
    Earth,
    Air,
    Energy,
}

#[derive(Debug, Clone, StructuralConvert, Serialize)]
#[convert(from(game::entities::Effect))]
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
impl From<game::entities::Card> for Card {
    fn from(value: game::entities::Card) -> Self {
        Self {
            element: (*value.element()).into(),
            effect: (*value.effect()).into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(game::query::PlayerPrivateQuery))]
pub struct PlayerPrivateQuery {
    pub id: Uuid,
    pub cards: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResult { //Гравець, що вибув з гри
    pub id: Uuid, //Ідентифікатор
    pub cards_had: usize, //Кількість карт які гравець мав за гру загалом
}
impl From<game::entities::PlayerResult> for PlayerResult {
    fn from(value: game::entities::PlayerResult) -> Self {
        Self {
            id: *value.id(),
            cards_had: *value.cards_had(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Turn {
    pub player: Option<Uuid>,
    pub card: Card,
}
impl From<game::entities::Turn> for Turn {
    fn from(value: game::entities::Turn) -> Self {
        Self {
            player: *value.player(),
            card: (*value.card()).into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(game::query::GameQuery))]
pub struct GameQuery {
    pub id: Uuid,
    pub card: Card,
    pub history: Vec<Turn>,
    pub turn: usize,
    pub turn_enforced_at: i64,
    pub direction: Direction,
    pub players: Vec<Uuid>,
    pub players_active: Vec<PlayerPrivateQuery>,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(game::query::GameNewTurnQuery))]
pub struct GameNewTurnQuery {
    pub id: Uuid,
    pub card: Card,
    pub turn: usize,
    pub turn_enforced_at: i64,
    pub direction: Direction,
    pub players_active: Vec<PlayerPrivateQuery>,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Serialize, StructuralConvert)]
#[convert(from(game::query::GameOverQuery))]
pub struct GameOverQuery {
    pub id: Uuid,
    pub card: Card,
    pub turn: usize,
    pub direction: Direction,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}