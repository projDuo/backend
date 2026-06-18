use poem::{handler, web::{ Data, Path, Json, Query }, Result };
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use crate::{ 
    AppState, domain::{game::{GameService}}, 
};
use super::super::auth::middleware::AuthenticatedUser;
use super::payloads::*;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub after: Option<Uuid>,
    pub limit: Option<u32>,
}

#[handler]
pub async fn history(
    Query(query): Query<HistoryQuery>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<Vec<GameHistoryPublic>>> {
    let history: Vec<GameHistoryPublic> = state
        .game_history
        .list_player_history(user.account_id, query.after, query.limit)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(history))
}

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