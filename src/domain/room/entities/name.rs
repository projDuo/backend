use super::Name;
use super::errors::*;

impl Default for Name {
    fn default() -> Self {
        Self("Room".to_string())
    }
}

impl Name {
    pub fn new(value: String) -> Result<Self, FieldError> { //Метод генерування ідентифікатора, що складається з 6 чисел
        if value.is_empty() || value.len() > 32 { return Err(FieldError::InvalidNameFormat); };

        let res = Self(value);
        Ok(res)
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}