use poem::{
    handler, web::{
        Data, Json, Path
    }, Result
};
use uuid::Uuid;
use std::sync::Arc;
use crate::{AppState, domain::savefiles::SavefilesService};
use super::payloads::*;
#[handler]
pub async fn get(
    Path(id): Path<Uuid>,
    state: Data<&Arc<AppState>>
) -> Result<Json<SavefilePublic>> {
    let savefile = state.as_ref().savefiles.load(id).await?;
    Ok(Json(savefile.into()))
}