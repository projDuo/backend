use poem::{
    handler, http::StatusCode, web::{
        Data, Json, Path
    }, Result
};
use std::sync::Arc;
use crate::{AppState, domain::{accounts::{AccountError, CreateAccountRequest, Login, Password}}};
use super::payloads::*;

use crate::domain::accounts::ports::AccountsService;

#[handler]
pub async fn get(
    Path(id): Path<String>,
    state: Data<&Arc<AppState>>
) -> Result<Json<AccountReadPublic>> {
    let account = state.as_ref().accounts.read_account_by_id_or_login(&id).await?
        .ok_or(AccountError::NotFound)?;
    Ok(Json(AccountReadPublic::from(account)))
}

#[handler]
pub async fn exists(
    Path(login): Path<String>,
    state: Data<&Arc<AppState>>
) -> Result<StatusCode, AccountError> {
    let is_taken = state.as_ref().accounts.is_login_taken(login).await?;
    if is_taken {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

#[handler]
pub async fn register(req: Json<Register>, state: Data<&Arc<AppState>>) -> Result<Json<AccountReadPublic>> {
    let login = Login::new(req.login.clone())?;
    let password = Password::new(req.password.clone().into()).await?;

    let cmd = CreateAccountRequest {
        login,
        password,
    };
    
    let res = state.as_ref().accounts.register(cmd).await?;
    Ok(Json(res.into()))
}
