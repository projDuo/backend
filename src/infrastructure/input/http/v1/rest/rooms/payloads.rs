use serde::{ Serialize, Deserialize };
use structural_convert::StructuralConvert;
use super::super::accounts::payloads::AccountReadPublic;
use crate::domain::room::{self as domain, FieldError, query::*, entities::*};
use uuid::Uuid;

fn limit() -> usize { 100 } //функція для задання значення за замовченням при десеріалізації

#[derive(Deserialize)]
pub struct RoomQuery { //структура, яка описує необхідні Query параметри у шляху, необхідна для пажировки списку кімнат
    #[serde(default)] //використання значення usize за замовченням - 0
    pub after: usize, //поле, яке указує після якої кількості кімнат необхідно почати формувати список для відповіді
    #[serde(default = "limit")] //виклик функції, якщо значення не отримано
    pub limit: usize, //поле, яке указує максимальну кількість кімнат які список для відповіді має містити
}

#[derive(Debug, StructuralConvert, Serialize)]
#[convert(from(domain::query::RoomListItem))]
pub struct RoomListItem {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub password: bool,
    pub owner: AccountReadPublic,
    pub max_players: u8,
    pub players: u8,
}

#[derive(Debug, Serialize, StructuralConvert)]
#[convert(from(PlayerBody))]
pub struct PlayerBodyResponse {
    pub is_ready: bool, 
    pub points: u64, 
}

#[derive(Debug, Serialize, StructuralConvert)]
#[convert(from(PlayerWithAccount))]
pub struct PlayerWithAccountResponse {
    #[serde(flatten)]
    pub account: AccountReadPublic,
    #[serde(flatten)]
    pub player: PlayerBodyResponse,
}

#[derive(Debug, Serialize, StructuralConvert)]
#[convert(from(Player))]
pub struct PlayerReady {
    pub is_ready: bool,
}

#[derive(Debug, Serialize, StructuralConvert)]
#[convert(from(domain::query::RoomPublic))]
pub struct RoomPublicResponse {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub password: bool,
    pub owner: Uuid,
    pub max_players: u8,
}

impl From<Room> for RoomPublicResponse {
    fn from(value: Room) -> Self {
        Self {
            id: value.id.get().to_string(),
            name: value.name.get().to_string(),
            is_public: *value.is_public.get(),
            password: value.password.is_some(),
            owner: value.owner,
            max_players: value.max_players.get()
        }
    }
}

#[derive(Debug, Serialize, StructuralConvert)]
#[convert(from(RoomWithPlayersEmbedded))]
pub struct RoomWithPlayersEmbeddedResponse {
    pub room: RoomPublicResponse,
    pub players: Vec<PlayerWithAccountResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: Option<String>,
    pub is_public: Option<bool>,
    pub password: Option<Option<String>>,
    pub max_players: Option<u8>,
}
impl CreateRoomRequest {
    pub fn try_into_room(&self, owner: Uuid) -> Result<domain::CreateRoomRequest, Vec<FieldError>> {
        let mut errors = Vec::new();

        let name = self.name.clone().map(
            |v| Name::new(v)
                .inspect_err(|e| errors.push(*e))
                .ok()
        ).unwrap_or_default();

        let is_public = self.is_public.map(
            |v| IsPublic::new(v)
        );

        let password = self.password.clone().map(
            |state|{
                state.map(|password| PasswordHash::new(password)
                    .inspect_err(|e| errors.push(*e))
                    .ok()
                )
            }
        ).unwrap_or_default();

        let max_players = self.max_players.clone().map(
            |v| MaxPlayers::new(v)
                .inspect_err(|e| errors.push(*e))
                .ok()
        ).unwrap_or_default();

        if !errors.is_empty() { return Err(errors); }
        
        let cmd = domain::commands::CreateRoomRequest::new(
            name,
            is_public,
            password,
            owner,
            max_players
        );
        Ok(cmd)
    }
} 

#[derive(Debug, Deserialize)]
pub struct UpdateRoomRequest {
    pub id: String,
    pub name: Option<String>,
    pub is_public: Option<bool>,
    pub password: Option<Option<String>>,
    pub owner: Option<Uuid>,
    pub max_players: Option<u8>,
}
impl TryInto<domain::commands::UpdateRoomRequest> for UpdateRoomRequest {
    type Error = Vec<FieldError>;
    
    fn try_into(self) -> Result<domain::commands::UpdateRoomRequest, Self::Error> {
        let mut errors = Vec::new();

        let name = self.name.map(
            |v| Name::new(v)
                .inspect_err(|e| errors.push(*e))
                .ok()
        ).unwrap_or_default();

        let is_public = self.is_public.map(
            |v| IsPublic::new(v)
        );

        let password = self.password.map(
            |state|{
                state.map(|password| PasswordHash::new(password)
                    .inspect_err(|e| errors.push(*e))
                    .ok()
                )
            }
        ).unwrap_or_default();

        let max_players = self.max_players.map(
            |v| MaxPlayers::new(v)
                .inspect_err(|e| errors.push(*e))
                .ok()
        ).unwrap_or_default();

        if !errors.is_empty() { return Err(errors); }
        
        let cmd = domain::commands::UpdateRoomRequest::new(
            self.id,
            name,
            is_public,
            password,
            self.owner,
            max_players
        );
        Ok(cmd)
    }
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest { //Структура, яка описує поля, запит на необхідні для підключення до кімнати повинен мати
    pub password: Option<String>,
}