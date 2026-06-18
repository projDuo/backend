pub mod accounts;
pub mod savefiles;
pub mod sessions;
pub mod auth;
pub mod game;
pub mod game_history;
pub mod room;
pub mod activity;
pub mod chat;
pub mod muted;

use thiserror::Error;
use tokio::sync::broadcast;

pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;


#[derive(Clone, Debug, Error)]
#[error("Internal repository error: {0}")]
pub struct InternalError(pub String);

pub trait EventBus<Event>: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<Event>;
    fn publish(&self, event: Event);
}