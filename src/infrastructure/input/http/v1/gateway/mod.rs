pub mod payloads;
pub mod events;

use futures_util::SinkExt;
use poem::{
    handler, http::StatusCode, web::{ websocket::{Message, WebSocket }, Data }, IntoResponse,
    Route, get
};
use serde_json;
use sea_orm::prelude::Uuid;
use std::{sync::Arc};
use tokio::sync::broadcast;
use futures_util::StreamExt;
use payloads::*;
use crate::AppState;
use crate::infrastructure::http::v1::rest::errors::ErrorCode;
use payloads::{Payload, EventOut};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(gateway))
}

#[handler]
pub async fn gateway(
    ws: WebSocket,
    state: Data<&Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let (sender, mut receiver) = broadcast::channel::<String>(12); //створення нового каналу
    //let mut receivers = sender.subscribe();
    let state = state.to_owned();
    Ok(
        ws.on_upgrade(move |mut socket| async move { //при перетворенні http з'єднання в gateway
            let (mut sink, mut stream) = socket.split(); //розділити з'єднання на вхідний та вихідний потік

            //Створення нового потоку для вхідних значень
            tokio::spawn(async move {
                let mut user_id: Option<Uuid> = None; //id користувача у цьому замиканні 

                while let Some(Ok(msg)) = stream.next().await { //Доки надходять повідомлення
                    let state = state.clone();
                    if let Message::Text(text) = msg { //Якщо повідомлення містить текст
                        let request = serde_json::from_str(&text); //Десеріалізувати з JSON

                        let event_out = if let Ok(Payload::EventIn(request)) = request { //якщо десеріалізація пройшла успішно
                            match request {
                                EventIn::Auth(auth::AuthEventIn::Identify(payload)) => //ідентифікація та авторизація акаунта за наданим токеном
                                    events::identify(state, payload, &mut user_id, sender.clone()).await,
                                EventIn::Auth(auth::AuthEventIn::Heartbeat) =>
                                    events::heartbeat(state, &user_id).await,
                            }
                        } else { 
                            Err(ErrorCode::new("DECLINED", "Unknown payload provided."))
                        }
                        .map_err(|e| EventOut::Error(e))
                        .unwrap_or_else(|e| e); //якщо десеріалізація повернула помилку, повернути її

                        let _ = sender.send(event_out.to_json_string()); //Надіслати payload клієнту
                    }
                }
            });

            //створити новий поток надсилання значень
            tokio::spawn(async move {
                //доки канал відкритий
                while let Ok(text) = receiver.recv().await {
                    //if text.contains("./") { continue };
                    let msg = Message::text(text);
                    if let Err(_) = sink.send(msg).await { //надіслати вхідний текст клієнту
                        break;
                    }
                }
            });

        })
    )
}