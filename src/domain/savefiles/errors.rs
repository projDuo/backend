use super::InternalRepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum SavefileError {
    #[error("Savefile not found")]
    NotFound,
    
    #[error("A savefile with this id already exists.")]
    Duplicate,
    
    #[error("Database is unavailable")]
    Infrastructure(#[from] InternalRepositoryError),
}