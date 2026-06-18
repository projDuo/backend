use poem::http::StatusCode;

use crate::domain::chat::ChatError;

use super::super::errors::{ErrorCode, RestError};
use super::super::errors::impl_api_error;

impl RestError for ChatError {
    fn rest_status(&self) -> StatusCode {
        match self {
            ChatError::ChannelNotFound => StatusCode::NOT_FOUND,
            ChatError::MessageNotFound => StatusCode::NOT_FOUND,
            ChatError::Unauthorized => StatusCode::FORBIDDEN,
            ChatError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn rest_error(&self) -> ErrorCode {
        let (error_code, message) = match self {
            ChatError::ChannelNotFound => ("CHAT_CHANNEL_NOT_FOUND", "The requested chat channel does not exist."),
            ChatError::MessageNotFound => ("CHAT_MESSAGE_NOT_FOUND", "The requested chat message does not exist."),
            ChatError::Unauthorized => ("CHAT_UNAUTHORIZED", "You don't have access to this message."),
            ChatError::Internal => ("INTERNAL_SERVER_ERROR", "Internal server error. Please try again later."),
        };

        ErrorCode::new(error_code, message)
    }
}

impl_api_error!(ChatError);
