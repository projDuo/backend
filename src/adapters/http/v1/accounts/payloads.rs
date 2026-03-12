use serde::Serialize;
use uuid::Uuid;
use crate::domain::{
    DateTimeWithTimeZone,
    accounts::Account,
    savefiles::Savefile,
};

#[derive(Debug, Serialize)]
pub struct AccountReadPublic {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<Account> for AccountReadPublic {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            login: value.login,
            display_name: value.display_name,
            created_at: value.created_at
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct AccountStats { //Структура, яка описує статистику гравця
    games_played: i64, //ігор зіграно
    points: i64, //очків
    cards_had: i64, //карт мав загалом
    wins: i32, //кількість виграшів
    loses: i32, //кількість виграшів
    max_points: i16, //максимальна кількість очків за гру
}

impl From<Savefile> for AccountStats {
    fn from(value: Savefile) -> Self { //перетворювач рядка в статистику
        Self {
            games_played: value.games_played,
            points: value.games_played,
            cards_had: value.cards_had,
            wins: value.wins,
            loses: value.loses,
            max_points: value.max_points,
        }
    }
}

#[derive(Serialize)]
struct AccountPublicFull {
    #[serde(flatten)]
    pub account: AccountReadPublic,
    #[serde(flatten)]
    pub savefile: AccountStats,
}