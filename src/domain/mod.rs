pub mod game;
pub mod accounts;

pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;
pub trait Identifiable {
    type Id;
    fn id(&self) -> Self::Id;
}