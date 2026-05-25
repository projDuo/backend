use super::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum RoomInternalEvent { 
    RoomRemovePlayer(Uuid)
}

#[derive(Debug, Clone)]
pub enum RoomEvent { //Список усіх можливих подій
    RoomPlayerNew(Player), //Подія приєднання гравця до кімнати
    RoomPlayerUpdate(Player), //Подія оновлення гравця кімнати
    RoomPlayerLeft(Uuid), //Подія виходу гравця з кімнати
    RoomCreate(RoomWithPlayersEmbedded), //Подія створення нової кімнати
    RoomUpdate(Room), //Подія оновлення кімнати
    RoomDelete(String), //Подія видалення кімнати
}