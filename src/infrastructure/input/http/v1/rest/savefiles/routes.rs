use poem::{
    handler, web::{
        Data, Json, Path
    }, Result
};
use uuid::Uuid;
use std::sync::Arc;
use crate::AppState;
use super::payloads::*;
#[handler]
pub async fn get(
    Path(id): Path<Uuid>,
    state: Data<&Arc<AppState>>
) -> Result<Json<SavefilePublic>> {
    let savefile = state.as_ref().savefiles.load(id).await?;
    Ok(Json(savefile.into()))
}

#[handler]
pub async fn get_the_best(
    state: Data<&Arc<AppState>>
) -> Result<Json<Vec<SavefilePublic>>> {
    let savefiles: Vec<SavefilePublic> = state.as_ref().savefiles.get_the_best().await?
        .iter()
        .map(|v| v.to_owned().into())
        .collect();
    Ok(Json(savefiles.into()))
}
