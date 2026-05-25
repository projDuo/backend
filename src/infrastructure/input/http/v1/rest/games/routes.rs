use poem::{handler, web::{ Data, Path, Json }, Result };
use uuid::Uuid;
use std::sync::Arc;
use crate::{ 
    AppState, domain::{game::{GameService}}, 
};
use super::super::auth::middleware::AuthenticatedUser;
use super::payloads::*;

#[handler]
pub async fn get( //функція доставання стану гри кімнати
    Path(id): Path<Uuid>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<GameQuery>> {
    let game = state.games.read_game_session_as_player(user.account_id, id).await?
        .into();
    Ok(Json(game))
}

#[handler]
pub async fn play(
    Path(Play { id, card_id }): Path<Play>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<PlayCard>> {
    let res = state.games.play_card(id, user.account_id, card_id).await?
        .into();
    Ok(Json(res))
}