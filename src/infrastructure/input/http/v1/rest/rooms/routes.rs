use poem::{ Result, handler, http::StatusCode, web::{ self, Data, Json, Path } };
use std::sync::Arc;
use uuid::Uuid;
use crate::{ 
    AppState, domain::{activity::ActivityService, room::{commands, errors::*, ports::*}},
};
use super::AuthenticatedUser;
use super::payloads::*;
use super::error_code::*;

#[handler]
pub async fn get_rooms_list(
    state: Data<&Arc<AppState>>,
    #[allow(unused)] //Is actually used for the auth lock
    user: Data<&AuthenticatedUser>,
    query: web::Query<RoomQuery>) -> Result<Json<Vec<RoomListItem>>> { //функція для формування списку
    let rooms = state.rooms.read_room_list(query.limit, query.after, query.search.clone()).await?
        .into_iter().map(Into::into).collect();
    Ok(Json(rooms)) //надсилання відповіді
}

#[handler]
pub async fn create(
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
    body: Json<CreateRoomRequest>,
) -> Result<Json<RoomWithPlayersEmbeddedResponse>> {
    let mut activity = state.activities.read_activity(user.account_id).await?;

    let mut cmd= vec_try!(body.0.try_into_room(user.account_id));
    let room: RoomWithPlayersEmbeddedResponse = state.rooms.create_room(cmd).await?
        .into();

    activity.room = Some(room.room.id.clone());
    let _ = state.activities.mark_activity(activity.into()).await;

    Ok(Json(room))
}

#[handler]
pub async fn update( //функція для оновлення параметрів кімнати
    Path(id): Path<String>,
    body: Json<UpdateRoomRequest>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<RoomPublicResponse>> {
    if user.account_id != state.rooms.get_room_owner(id).await?
        { return Err(RoomError::NotAnOwner.into()) }

    let cmd: commands::UpdateRoomRequest = vec_try!(body.0.try_into());
    let room = state.rooms.update_room(cmd).await?.into();

    Ok(Json(room))
}

#[handler]
pub async fn get(
    Path(id): Path<String>,
    state: Data<&Arc<AppState>>,
    _user: Data<&AuthenticatedUser>,
) -> Result<Json<RoomWithPlayersEmbeddedResponse>> {
    let room = state.rooms.read_room(id).await?;
    Ok(Json(room.into()))
}

#[handler]
pub async fn join( //функція для приєднання до кімнати
    Path(id): Path<String>,
    body: Json<JoinRoomRequest>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<RoomWithPlayersEmbeddedResponse>> {
    let mut activity = state.activities.read_activity(user.account_id).await?;
    
    let player = state.rooms.add_room_player(user.account_id, id, body.0.password).await?;
    let room: RoomWithPlayersEmbeddedResponse = state.rooms.read_room(player.room_id().to_string()).await
        .expect("Player's room supposed to exist")
        .into();

    activity.room = Some(room.room.id.clone());
    let _ = state.activities.mark_activity(activity.into()).await;

    Ok(Json(room)) 
}


#[handler]
pub async fn leave( //функція для виходу з кімнати
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<StatusCode> {
    let mut activity = state.activities.read_activity(user.account_id).await?;

    state.rooms.remove_room_player(user.account_id).await?;

    activity.room = None;
    let _ = state.activities.mark_activity(activity.into()).await;

    Ok(StatusCode::OK)
}

#[handler]
pub async fn ready( //функція для перемикання готовності гравця
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<PlayerReady>> {
    let mut player = state.rooms.read_player_by_id(user.account_id).await?;
    player.is_ready = !player.is_ready;
    let player = state.rooms.update_room_player(player.into()).await?.into();

    Ok(Json(player))
}

#[handler]
pub async fn kick(
    Path((id, player_id)): Path<(String, Uuid)>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<StatusCode> {
    state.rooms.kick_room_player(user.account_id, id, player_id).await?;
    Ok(StatusCode::OK)
}