use serde::{Deserialize, Serialize};
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
            login: value.login.to_string(),
            display_name: value.display_name.map(|v| v.to_string()),
            created_at: value.created_at
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountStats { //Структура, яка описує статистику гравця
    games_played: u64, //ігор зіграно
    points: u64, //очків
    cards_had: u64, //карт мав загалом
    wins: u32, //кількість виграшів
    loses: u32, //кількість виграшів
    max_points: u16, //максимальна кількість очків за гру
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
pub struct AccountPublicFull {
    #[serde(flatten)]
    pub account: AccountReadPublic,
    #[serde(flatten)]
    pub savefile: AccountStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Register { //структура, яка задає які поля запит на регістрацію повинен містити
    pub login: String, //логін
    pub password: String, //пароль
}