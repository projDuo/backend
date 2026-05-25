use super::IsPublic;

impl Default for IsPublic {
    fn default() -> Self {
        Self(false)
    }
}

impl IsPublic {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
    
    pub fn get(&self) -> &bool {
        &self.0
    }
}