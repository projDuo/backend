use crate::domain::game::rooms::{Room, Error};

use uuid::Uuid;

struct RoomCreate {
    name: String, //Назва
    pub is_public: bool, //Видимість в списку кімнат
    password: Option<String>, //Пароль
    owner: Uuid, //Власник
    max_players: usize, //Максимальна кількість гравців
}

impl RoomCreate {
    pub fn new(name: String, is_public: bool, password: Option<String>, owner: Uuid, max_players: usize) -> Self {
        Self {
            name,
            is_public,
            password,
            owner,
            max_players,
        }
    }
}

impl TryFrom<RoomCreate> for Room {
    type Error = Error<'static>;

    fn try_from(value: RoomCreate) -> Result<Self, Self::Error> {
        Self::create(
            value.name,
            value.is_public,
            value.password,
            value.owner,
            value.max_players,
        )
    }
}

struct RoomUpdate {
    id: String, //Ідентифікатор кімнати
    name: String, //Назва
    pub is_public: bool, //Видимість в списку кімнат
    password: Option<String>, //Пароль
    owner: Uuid, //Власник
    max_players: usize, //Максимальна кількість гравців
}

impl From<Room> for RoomUpdate {
    fn from(value: Room) -> Self {
        Self {
            id: value.id().to_string(),
            name: value.name().to_string(),
            is_public: value.is_public,
            password: value.password().clone(),
            owner: *value.owner(),
            max_players: *value.max_players(),
        }
    }
}