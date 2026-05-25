use super::MaxPlayers;
use super::errors::*;

impl Default for MaxPlayers {
    fn default() -> Self {
        Self(2)
    }
}

impl MaxPlayers {
    pub fn new(value: u8) -> Result<Self, FieldError> {
        if value < 2 || value > 4 { return Err(FieldError::OutOfRangeMaxPlayers); }
        
        let res = Self(value);
        Ok(res)
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}