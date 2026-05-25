use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ActivityInternalEvent {
    UserJoinedRoom{room_id: String, user_id: Uuid},
    UserLeftRoom{user_id: Uuid},
    UserJoinedGame{game_id: Uuid, user_id: Uuid},
    UserLeftGame{user_id: Uuid},
}

#[derive(Debug, Clone)]
pub enum ActivityEvent {
    JoinedRoom{room_id: String},
    LeftRoom,
    JoinedGame{game_id: Uuid},
    LeftGame,
}