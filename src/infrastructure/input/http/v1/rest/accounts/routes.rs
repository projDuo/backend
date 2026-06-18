use poem::{
    handler, http::StatusCode, web::{
        Data, Json, Path
    }, Error, Result
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{AppState, domain::{accounts::{AccountError, CreateAccountRequest, Login, Password, DisplayName, UpdateAccountRequest}}};
use super::payloads::*;
use super::super::auth::middleware::AuthenticatedUser;

use crate::domain::accounts::ports::AccountsService;

#[handler]
pub async fn get(
    Path(id): Path<String>,
    state: Data<&Arc<AppState>>
) -> Result<Json<AccountReadPublic>> {
    let account = state.as_ref().accounts.read_account_by_id_or_login(&id).await?;
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

#[handler]
pub async fn update(
    Path(id): Path<String>,
    req: Json<UpdateAccount>,
    user: Data<&AuthenticatedUser>,
    state: Data<&Arc<AppState>>,
) -> Result<Json<AccountReadPublic>> {
    let account_id = Uuid::try_parse(&id)
        .map_err(|_| Error::from_string("Invalid account id", StatusCode::BAD_REQUEST))?;

    if account_id != user.account_id {
        return Err(Error::from_string("Forbidden", StatusCode::FORBIDDEN));
    }

    let login = match req.login.clone() {
        Some(value) => Some(Login::new(value)?),
        None => None,
    };

    let password = match req.password.clone() {
        Some(value) => Some(Password::new(value.into()).await?),
        None => None,
    };

    let display_name = match req.display_name.clone() {
        Some(value) => Some(DisplayName::new(value)?),
        None => None,
    };

    let cmd = UpdateAccountRequest {
        id: account_id,
        login,
        password,
        display_name,
    };

    let res = state.as_ref().accounts.update_account(cmd).await?;
    Ok(Json(res.into()))
}
