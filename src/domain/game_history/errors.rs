use crate::domain::InternalError;

#[derive(Debug, thiserror::Error)]
pub enum GameHistoryError {
    #[error("Game history entry already exists")]
    Duplicate,

    #[error("Database is unavailable")]
    Infrastructure(#[from] InternalError),
}
