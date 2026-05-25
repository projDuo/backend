use super::errors::*;
use std::sync::LazyLock;
use regex::Regex;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Algorithm, Argon2, Params, Version
};
use secrecy::{ExposeSecret, SecretString};

static PASSWORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z\d@$!%*#?&]{8,}$").unwrap()
});

#[derive(Clone, PartialEq, Debug)]
pub struct Password(String);

impl Password {
    pub async fn new(raw: SecretString) -> Result<Self, WeakPassword> {
        let s = raw.expose_secret();
        if s.len() < 8 {
            return Err(WeakPassword);
        }
        if !PASSWORD_REGEX.is_match(s) {
            return Err(WeakPassword);
        }
        let has_letter = s.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = s.chars().any(|c| c.is_ascii_digit());
        if !(has_letter && has_digit) {
            return Err(WeakPassword);
        }

        let hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            
            let params = Params::new(16384, 2, 1, None).expect("Invalid argon2 params");
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            
            argon2
                .hash_password(raw.expose_secret().as_bytes(), &salt)
                .map(|h| h.to_string())
                .expect("PasswordHashingError")
        })
        .await
        .expect("PasswordNewHashThreadError");
        
        Ok(Self(hash))
    }

    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    pub async fn verify(&self, raw: SecretString) -> bool {
        let stored_hash = self.0.clone();

        tokio::task::spawn_blocking(move || {
            let parsed_hash = match PasswordHash::new(&stored_hash) {
                Ok(h) => h,
                Err(_) => return false,
            };
            
            Argon2::default()
                .verify_password(raw.expose_secret().as_bytes(), &parsed_hash)
                .is_ok()
        })
        .await
        .unwrap_or(false) 
    }
}

impl ToString for Password {
    fn to_string(&self) -> String {
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
