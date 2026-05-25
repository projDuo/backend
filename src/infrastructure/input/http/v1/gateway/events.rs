
use sea_orm::{prelude::Uuid};
use serde::Serialize;
use tokio::sync::{broadcast::{ Receiver, Sender, error::RecvError }};
use std::sync::Arc;
use crate::{
    AppState,
    domain::{
        activity::{
            ActivityEventBus,
            ActivityService,
            MarkActivityCommand,
            events::ActivityEvent
        },
        auth::AuthService,
        game::{
            GameEventBus,
            GameEvents
        }, 
        room::{
            RoomEvent,
            RoomEventBus
        }
    },
    infrastructure::http::v1::{
        errors::ErrorCode,
        payloads::{
            EventOut,
            auth::{
                AuthEventOut,
                output::Authorized
            },
            game::GameEventOut,
            room::RoomEventOut,
            activity::ActivityEventOut,
        }
    }
};
use super::payloads::auth;
use crate::domain::accounts::ports::AccountsService;
use crate::application::activity::ACTIVITY_HEARTBEAT_SECS;

//Receive

pub async fn identify( //Функція яка ідентифікує акаунт за токеном, авторизує та відновлює сесію
    state: Arc<AppState>,
    payload: auth::input::Identify,
    store_identity_in: &mut Option<Uuid>,
    tx: Sender<String>,
) -> Result<EventOut, ErrorCode> {
    if store_identity_in.is_some() { return Err(
        ErrorCode::new("ALREADY_IDENTIFIED", "Already identified.")
    )}

    let token = payload.token;
    let uuid = state.auth.verify(token.into()).await?.account_id; //Перевірка токену на валідність та доставання id акаунту, повернення помилки у разі невірного токену
    *store_identity_in = Some(uuid);

    let mark_activity_cmd = MarkActivityCommand::new(uuid);
    let activity = state.activities.mark_activity(mark_activity_cmd).await?;
    let account= state.accounts.read_account(uuid).await?;
    
    let mut activity_event_bus = state.activity_event_bus.subscribe(uuid)
        .expect("Channel is supposed to be opened here by mark_activity");

    let activity_shutdown_tx = Sender::new(1);
    let room_shutdown_tx = Sender::new(1);
    let game_shutdown_tx = Sender::new(1);

    if let Some(v) = &activity.room {
        if let Some(rx) = state.room_event_bus.subscribe(v.to_string()) {
            listen::<RoomEvent, RoomEventOut>(rx, tx.clone(), room_shutdown_tx.subscribe());
        };
    };
    if let Some(_v) = &activity.game {
        if let Some(rx) = state.game_event_bus.subscribe(uuid) {
            listen::<GameEvents, GameEventOut>(rx, tx.clone(), game_shutdown_tx.subscribe());
        }
    }

    listen::<ActivityEvent, ActivityEventOut>(activity_event_bus.resubscribe(), tx.clone(), activity_shutdown_tx.subscribe());

    tokio::spawn(async move {
        let _shutdown_tx = activity_shutdown_tx;
        let user_id = uuid.clone();
        while let Ok(event) =  activity_event_bus.recv().await {
            match event {
                ActivityEvent::JoinedRoom { room_id } => {
                    let _ = room_shutdown_tx.send(());
                    let Some(rx) = state.room_event_bus.subscribe(room_id)
                        else { return };
                    listen::<RoomEvent, RoomEventOut>(rx, tx.clone(), room_shutdown_tx.subscribe());
                },
                ActivityEvent::LeftRoom => {
                    let _ = room_shutdown_tx.send(());
                },
                ActivityEvent::JoinedGame { game_id } => {
                    tracing::info!("huhuh: {} {}", game_id, user_id);
                    let _ = game_shutdown_tx.send(());
                    let Some(rx) = state.game_event_bus.subscribe(user_id)
                        else { continue; };
                    tracing::info!("no?");
                    listen::<GameEvents, GameEventOut>(rx, tx.clone(), game_shutdown_tx.subscribe());
                },
                ActivityEvent::LeftGame => {
                    let _ = game_shutdown_tx.send(());
                }
            }
        }
    });

    let res = EventOut::Auth(
        AuthEventOut::Authorized(
            Authorized::new(
                ACTIVITY_HEARTBEAT_SECS as u64,
                account.into(),
                Some(activity.into()),
            )
        )
    );

    Ok(res)
}

pub async fn heartbeat(
    state: Arc<AppState>,
    user_id: &Option<Uuid>,
) -> Result<EventOut, ErrorCode> {
    let Some(uuid) = user_id else {
        return Err(ErrorCode::new(
            "NOT_IDENTIFIED",
            "Identify before sending heartbeat.",
        ));
    };

    let _ = state
        .activities
        .mark_activity(MarkActivityCommand::new(*uuid))
        .await
        .map_err(|_| {
            ErrorCode::new("INTERNAL_SERVER_ERROR", "Failed to refresh activity.")
        })?;

    Ok(EventOut::Auth(AuthEventOut::HeartbeatAck))
}

fn listen<Event, EventOut>(
    mut rx: Receiver<Event>, 
    tx: Sender<String>,
    mut shutdown_rx: Receiver<()>
)
where
    Event: Into<EventOut> + Clone + Send + 'static,
    EventOut: Serialize + Send + 'static, {
    tokio::spawn(async move { loop { tokio::select! {
        res = rx.recv() => {
            match res {
                Ok(event) => {
                    let event_out: EventOut = event.into();
                    let json = match serde_json::to_string(&event_out) {
                        Ok(serialized) => serialized,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to serialize event");
                            continue; 
                        }
                    };
                    if tx.send(json).is_err() {
                        break;
                    };
                },
                Err(RecvError::Lagged(_)) => { continue; }
                Err(RecvError::Closed) => {
                    tracing::debug!("Event bus closed. Dropping listener.");
                    break;
                }
            }
        }

        _ = shutdown_rx.recv() => {
            break;
        }
    }}});
}