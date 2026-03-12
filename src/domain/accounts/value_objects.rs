use super::errors::*;
use std::sync::LazyLock;
use regex::Regex;

#[derive(PartialEq)]
pub struct Password(String);

impl Password {
    pub fn new(raw: String) -> Result<Self, WeakPassword> {
        if raw.len() < 6 {
            return Err(WeakPassword);
        }
        Ok(Self(raw))
    }

    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    pub fn hash(&self) -> String {
        self.0.clone()
    }
}

#[derive(PartialEq)]
pub struct Login(String);
static LOGIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9]+$").expect("Invalid regex pattern")
});

impl Login {
    pub fn new(value: String) -> Result<Self, InvalidLogin> {
        if Self::is_valid(value.as_str()) {
            Ok(Self(value))
        } else {
            Err(InvalidLogin)
        }
    }

    pub fn from_raw(value: String) -> Self {
        Self(value)
    }

    fn is_valid(value: &str) -> bool {
        LOGIN_REGEX.is_match(value)
    }
}

impl ToString for Login {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(PartialEq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl ToString for DisplayName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
