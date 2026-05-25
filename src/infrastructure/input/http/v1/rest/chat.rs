use poem::{handler, web::{Data, Json}, http::StatusCode};
use crate::infrastructure::rest::v1::auth::middleware::AuthenticatedUser;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::AppState;
use crate::infrastructure::gateway::payloads::*; 

#[derive(serde::Deserialize)]
pub struct SendMessage { pub content: String }

#[derive(serde::Deserialize)]
pub struct ReportPlayer { pub target_id: Uuid, pub content: String }

#[handler]
pub async fn send_message(
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    rooms_ptr: Data<&Arc<RwLock<crate::Rooms>>>,
    req: Json<SendMessage>,
) -> Result<StatusCode, StatusCode> {
    let players = players_ptr.read().await;
    let sender = players.get(&user.account_id).ok_or(StatusCode::UNAUTHORIZED)?;


    if sender.is_muted { return Err(StatusCode::FORBIDDEN); }

    if let Some(room_id) = &sender.room {
        let rooms = rooms_ptr.read().await;
        if let Some(room) = rooms.get(room_id) {
            let _ = room.announce(Payload::ChatBroadcast {
                sender_id: *sender.uuid(),
                content: req.content.clone(),
            }.to_json_string());
        }
    }
    Ok(StatusCode::OK)
}

#[handler]
pub async fn report_player(
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    req: Json<ReportPlayer>,
) -> Result<StatusCode, StatusCode> {
    let players = players_ptr.read().await;
    let reporter = players.get(&user.account_id).ok_or(StatusCode::UNAUTHORIZED)?;

    let alert = Payload::ReportAlert {
        reporter_id: *reporter.uuid(),
        offender_id: req.target_id,
        content: req.content.clone(),
    }.to_json_string();

    for p in players.iter() {
        if p.is_moderator {
            let _ = p.sender.send(alert.clone());
        }
    }
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct MutePlayerReq {
    pub target_id: Uuid,
}

#[handler]
pub async fn mute_player(
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    req: Json<MutePlayerReq>,
) -> Result<StatusCode, StatusCode> {
    let mut players = players_ptr.write().await;
    
    let is_mod = players.get(&user.account_id).map(|p| p.is_moderator).unwrap_or(false);
    if !is_mod {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(mut target_player) = players.take(&req.target_id) {
        target_player.is_muted = true;
        players.insert(target_player);
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND) 
    }
}