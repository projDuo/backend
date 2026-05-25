use uuid::Uuid;
use getset::Getters;

#[derive(Getters, Clone)]
pub struct Savefile {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub")]
    pub games_played: u64,
    #[getset(get = "pub")]
    pub points: u64,
    #[getset(get = "pub")]
    pub cards_had: u64,
    #[getset(get = "pub")]
    pub wins: u32,
    #[getset(get = "pub")]
    pub loses: u32,
    #[getset(get = "pub")]
    pub max_points: u16,
}

impl Savefile {
    pub fn new(
        id: Uuid,
        games_played: u64,
        points: u64,
        cards_had: u64,
        wins: u32,
        loses: u32,
        max_points: u16,
    ) -> Self {
        Self {
            id,
            games_played,
            points,
            cards_had,
            wins,
            loses,
            max_points
        }
    }
} 