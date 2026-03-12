

pub struct SessionToken(String);

impl SessionToken {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ToString for SessionToken {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}