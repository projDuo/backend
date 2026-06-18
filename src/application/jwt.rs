use jsonwebtoken::{decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};

use crate::domain::{InternalError, auth::*, sessions::SessionError};

//https://www.iana.org/assignments/jwt/jwt.xhtml
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,          // Account ID
    pub sid: Uuid,          // Session ID
    pub exp: usize,         // Expiration timestamp
    pub iat: usize,         // Issued at
}

#[derive(Clone)]
pub struct Service {
    secret: String,
    refresh_expires_after: Duration,
    access_expires_after: Duration, 
}

impl Service {
    pub fn new(
        secret: impl Into<String>,
        refresh_expires_after: Duration,
        access_expires_after: Duration,
    ) -> Self {
        Self {
            secret: secret.into(),
            refresh_expires_after,
            access_expires_after,
        }
    }
}

impl TokenProvider for Service {
    fn refresh_expires_after(&self) -> Duration {
        self.refresh_expires_after
    }

    fn access_expires_after(&self) -> Duration {
        self.access_expires_after
    }
    
    fn generate_pair(&self, account_id: Uuid, session_id: Uuid) -> Result<TokenPair, AuthError> {
        let access_token = self.generate_token(account_id, session_id)?;

        let refresh_token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();

        Ok(TokenPair { access_token, refresh_token })
    }

    fn generate_token(&self, account_id: Uuid, session_id: Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + self.access_expires_after;

        let claims = Claims {
            sub: account_id,
            sid: session_id,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| InternalError(e.to_string()).into()) 
    }

    fn verify_token(&self, token: SecretString) -> Result<TokenData, AuthError> {
        let token_data = decode::<Claims>(
            token.expose_secret(),
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|err| match err.kind() {
            ErrorKind::ExpiredSignature => SessionError::Expired,
            _ => SessionError::Invalid,
        })?;

        Ok(TokenData {
            account_id: token_data.claims.sub,
            session_id: token_data.claims.sid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn generate_and_verify_pair() {
        let svc = Service::new("test-secret", chrono::Duration::minutes(5), chrono::Duration::minutes(10));
        let account_id = uuid::Uuid::now_v7();
        let session_id = uuid::Uuid::now_v7();

        let pair = svc.generate_pair(account_id, session_id).expect("generate_pair");
        let token_data = svc.verify_token(SecretString::new(pair.access_token.into_boxed_str())).expect("verify");
        assert_eq!(token_data.account_id, account_id);
        assert_eq!(token_data.session_id, session_id);
    }
}