use poem::{
    handler, web::{
        Data, Json, Path
    },
    http::StatusCode,
};
use std::sync::Arc;
use crate::AccountsService;
use super::{payloads, errors::Error};

#[handler]
pub async fn get(
    Path(id): Path<String>,
    accounts_service: Data<&Arc<AccountsService>>
) -> Result<Json<payloads::AccountReadPublic>, Error> {
    let account = accounts_service.as_ref().read_by_id_or_login(&id).await?;
    Ok(Json(payloads::AccountReadPublic::from(account)))
}

#[handler]
pub async fn exists(
    Path(login): Path<String>,
    accounts_service: Data<&Arc<AccountsService>>
) -> Result<StatusCode, Error> {
    let is_taken = accounts_service.as_ref().login_taken(login.as_str()).await?;
    if is_taken {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
