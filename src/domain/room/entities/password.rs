use super::errors::*;

use super::PasswordHash;
impl PasswordHash {
    pub fn new(value: String) -> Result<Self, FieldError> { //Сеттер пароля
        if value.len() > 32
        || value.is_empty() {
            return Err(FieldError::InvalidPasswordFormat);
        }

        let res = Self::new_unchecked(value);
        Ok(res)
    }

    pub fn new_unchecked(value: String) -> Self {
        let hash = sha256::digest(value);
        Self(hash)
    }
}