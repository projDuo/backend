use super::InternalError;

#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("The requested activity is not found.")]
    NotFound,
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InternalError),
}