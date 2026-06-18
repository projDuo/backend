use getset::Getters;
use uuid::Uuid;
use super::errors;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct RoomId(String);
pub mod room_id;

#[derive(Debug, Clone)]
pub struct Name(String);
pub mod name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);
pub mod password;

#[derive(Debug, Copy, Clone)]
pub struct MaxPlayers(u8); 
pub mod max_players;

#[derive(Debug, Clone)]
pub struct Room { //Структура, що описує кімнату
    pub id: RoomId, //Ідентифікатор
    pub name: Name, //Назва
    pub is_public: bool, //Видимість в списку кімнат
    pub password: Option<PasswordHash>, //Пароль
    pub owner: Uuid, //Власник
    pub max_players: MaxPlayers, //Максимальна кількість гравців
    //players: HashSet<Uuid>, //Посилання на таблицю гравців кімнати
    //game: Uuid, //посилання на об'єкт гри
}
pub mod room;

#[derive(Debug, Clone, Getters)]
pub struct Player { //Структура, що описує гравця у кімнаті
    #[getset(get = "pub")]
    pub id: Uuid, //Ідентифікатор
    #[getset(get = "pub")]
    room_id: String, //Room
    pub is_ready: bool, //Готовність
}
pub mod player;
