use uuid::Uuid;

use super::entities::*;

#[derive(Debug, Clone)]
pub struct PlayerPrivateQuery {
    pub id: Uuid,
    pub cards: u8,
}
impl From<Player> for PlayerPrivateQuery {
    fn from(value: Player) -> Self {
        Self { id: *value.id(), cards: value.cards().len() as u8 }
    }
}

#[derive(Debug, Clone)]
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
impl GameQuery {
    pub fn from_game(
        game: Game,
        players_active: Vec<PlayerPrivateQuery>,
        leaderboard: Vec<PlayerResult>,
        hand: Option<Vec<Card>>
    ) -> Self {
        Self { 
            id: *game.id(),
            card: *game.history.last().expect("Must contain at least one").card(),
            history: game.history.clone(),
            turn: game.turn,
            turn_enforced_at: game.turn_enforced_at,
            direction: game.direction,
            players: game.players().clone(),
            players_active,
            leaderboard,
            hand
        }
    }
}

#[derive(Debug, Clone)]
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
impl GameNewTurnQuery {
    pub fn from_game(
        game: Game,
        players_active: Vec<PlayerPrivateQuery>,
        leaderboard: Vec<PlayerResult>,
        hand: Option<Vec<Card>>
    ) -> Self {
        Self { 
            id: *game.id(),
            card: *game.history.last().expect("Must contain at least one").card(),
            turn: game.turn,
            turn_enforced_at: game.turn_enforced_at,
            direction: game.direction,
            players_active,
            leaderboard,
            hand
        }
    }
}


#[derive(Debug, Clone)]
pub struct GameOverQuery {
    pub id: Uuid,
    pub card: Card,
    pub turn: usize,
    pub direction: Direction,
    pub leaderboard: Vec<PlayerResult>,
    pub hand: Option<Vec<Card>>,
}
impl GameOverQuery {
    pub fn from_game(
        game: Game,
        leaderboard: Vec<PlayerResult>,
        hand: Option<Vec<Card>>
    ) -> Self {
        Self { 
            id: *game.id(),
            card: *game.history.last().expect("Must containe at least one").card(),
            turn: game.turn,
            direction: game.direction,
            leaderboard,
            hand
        }
    }
}

pub enum PlayCard {
    NextTurn(GameNewTurnQuery),
    GameOver(GameOverQuery)
}