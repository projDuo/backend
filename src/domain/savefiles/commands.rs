use uuid::Uuid;

pub struct InitSavefileRequest {
    pub id: Uuid
}

pub struct UpdateSavefileRequest {
    pub id: Uuid,
    pub games_played: Option<u64>,
    pub points: Option<u64>,
    pub cards_had: Option<u64>,
    pub wins: Option<u32>,
    pub loses: Option<u32>,
    pub max_points: Option<u16>,
}