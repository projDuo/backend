use futures::TryFutureExt;
use poem::{
    handler, http::StatusCode, web::{
        Data, Json
    }, Request, Response
};
use sea_orm::{prelude::Uuid, DatabaseConnection, DbErr, TryInsertResult};
use std::{ops::Deref, sync::Arc};
use crate::{AccountsService, AppState, domain::auth::{AuthError, TokenPair}};
use sha256;
use super::payloads::*;
use crate::domain::auth::ports::AuthService;

#[handler]
pub async fn login(req: Json<Login>, state: Data<&Arc<AppState>>) -> Result<Json<TokenPairPayload>, AuthError> {
    let pair = state.as_ref().auth.create(req.login.clone(), req.password.clone().into()).await?;
    Ok(Json(pair.into()))
}

#[handler]
pub async fn logout(req: &Request, state: Data<&Arc<AppState>>) -> Result<StatusCode, StatusCode> {
    state.auth.revoke(token_data)

    let HashedToken = Uuid::parse_str(req.header("authorization").ok_or(StatusCode::UNAUTHORIZED)?)
        .map_err(|_| StatusCode::BAD_REQUEST)?; //повернути помилку якщо токен не вказаний в запиті 
    delete(db, HashedToken).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).await?; //виклик функції для видалення сесії за токеном
    Ok(StatusCode::OK)
}

#[handler]
pub async fn logout_all(req: &Request, db: Data<&Arc<DatabaseConnection>>) -> Result<StatusCode, StatusCode> {
    let db = db.deref().as_ref();
    let HashedToken = Uuid::parse_str(req.header("authorization").ok_or(StatusCode::UNAUTHORIZED)?)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    delete_all_of_account(db, HashedToken).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).await?; //виклик функції для видалення усіх сесії за токеном
    Ok(StatusCode::OK)
}