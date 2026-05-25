use poem::{Endpoint, Middleware, Request, Result, Error};
use poem::http::StatusCode;
use std::sync::Arc;

use crate::domain::auth::{AuthService, TokenData};
use crate::domain::sessions::{SessionError, SessionsRepository};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub account_id: Uuid,
    pub session_id: Uuid,
}

impl From<AuthenticatedUser> for TokenData {
    fn from(value: AuthenticatedUser) -> Self {
        Self::new(value.account_id, value.session_id)
    }
}

pub struct AuthMiddleware {
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>,
}

impl AuthMiddleware {
    pub fn new(
        auth_service: Arc<dyn AuthService + Send + Sync>,
        sessions_repo: Arc<dyn SessionsRepository + Send + Sync>,
    ) -> Self {
        Self { auth_service, sessions_repo }
    }
}

impl<E: Endpoint> Middleware<E> for AuthMiddleware {
    type Output = AuthInterceptor<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthInterceptor {
            inner_endpoint: ep,
            auth_service: self.auth_service.clone(),
            sessions_repo: self.sessions_repo.clone(),
        }
    }
}

pub struct AuthInterceptor<E> {
    inner_endpoint: E,
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>,
}

impl<E: Endpoint> Endpoint for AuthInterceptor<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok());

        let jwt_string = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..],
            _ => return Err(Error::from_string("Missing or invalid Authorization header", StatusCode::UNAUTHORIZED)),
        };

        let token_data = self.auth_service.verify(jwt_string.into()).await?;


        let _ = self.sessions_repo.find_by_id(token_data.session_id).await?
            .ok_or(SessionError::Invalid)?;

        let auth_user = AuthenticatedUser {
            account_id: token_data.account_id,
            session_id: token_data.session_id,
        };
        req.extensions_mut().insert(auth_user);

        self.inner_endpoint.call(req).await
    }
}