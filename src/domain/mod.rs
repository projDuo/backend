pub mod game;
pub mod accounts;
pub mod savefiles;

pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;
pub trait Identifiable {
    type Id;
    fn id(&self) -> Self::Id;
}

pub trait ErrorType {
    type Error;
}