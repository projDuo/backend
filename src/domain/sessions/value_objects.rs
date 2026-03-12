use chrono::{DateTime, Duration, Utc};

use crate::domain::DateTimeWithTimeZone;

pub struct ExpiresAt(DateTimeWithTimeZone);
impl ExpiresAt {
    pub fn new(expires_after: Duration) -> Self {
        let expires_at = chrono::Utc::now() + expires_after;
        Self(expires_at.into())
    }

    pub fn as_utc(&self) -> DateTime<Utc> {
        self.0.to_utc()
    }
}

impl Into<DateTimeWithTimeZone> for ExpiresAt {
    fn into(self) -> DateTimeWithTimeZone {
        self.0
    }
}

impl From<DateTimeWithTimeZone> for ExpiresAt {
    fn from(value: DateTimeWithTimeZone) -> Self {
        Self(value)
    }
}

pub struct HashedToken(String);
impl HashedToken {
    pub fn new(raw: String) -> Self {
        Self(sha256::digest(raw))
    }
}

impl From<String> for HashedToken {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl ToString for HashedToken {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}