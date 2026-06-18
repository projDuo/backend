
#[derive(Debug, thiserror::Error, Clone)]
pub enum ChatError {
    #[error("Chat channel not found.")]
    ChannelNotFound,
    #[error("Chat message not found.")]
    MessageNotFound,
    #[error("You don't have access to this message.")]
    Unauthorized,
    #[error("Internal chat error.")]
    Internal,
}