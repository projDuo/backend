use poem::{
    handler, http::StatusCode, web::{
        Data, Json
    },
};
use std::sync::Arc;
use crate::{AppState, domain::auth::AuthError};
use super::super::auth::middleware::AuthenticatedUser;
use super::payloads::*;
use crate::domain::auth::ports::AuthService;

#[handler]
pub async fn login(req: Json<Login>, state: Data<&Arc<AppState>>) -> Result<Json<TokenPairPayload>, AuthError> {
    let pair = state.as_ref().auth.create(req.login.clone(), req.password.clone().into()).await?;
    Ok(Json(pair.into()))
}

#[handler]
pub async fn refresh(req: Json<Refresh>, state: Data<&Arc<AppState>>) -> Result<Json<TokenPairPayload>, AuthError> {
    let pair = state.auth.refresh(req.refresh_token.clone().into()).await?;
    Ok(Json(pair.into()))
}

#[handler]
pub async fn logout(state: Data<&Arc<AppState>>, user: Data<&AuthenticatedUser>) -> Result<StatusCode, AuthError> {
    let _ = state.auth.revoke(user.clone().into()).await?;
    Ok(StatusCode::OK)
}

#[handler]
pub async fn logout_all(state: Data<&Arc<AppState>>, user: Data<&AuthenticatedUser>) -> Result<StatusCode, AuthError> {
    let _ = state.auth.revoke_all(user.clone().into()).await?;
    Ok(StatusCode::OK)
}