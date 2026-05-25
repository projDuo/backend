use poem::{handler, http::StatusCode, web::{ Data, Path }, Response };
use serde::Deserialize;
use tokio::sync::RwLock;
use std::{ ops::Deref, sync::Arc };
use crate::{ 
    AppState, Rooms, domain::{game::gameplay::Ok, savefiles::SavefilesService}, runtime_storage::Table
};
use super::prelude;
use futures::executor;
use super::super::auth::middleware::AuthenticatedUser;

#[handler]
pub async fn get( //функція доставання стану гри кімнати
    Path(id): Path<String>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    rooms_ptr: Data<&Arc<RwLock<Rooms>>>,
) -> Result<Response, StatusCode> {
    let (_players, rooms, _player) =
        prelude(state.as_ref(), user, players_ptr.deref(), rooms_ptr.deref()).await?; //виклик функції бронювання даних за показниками
    //пошук кімнати в таблиці кімнат, обробка помилок та надсилання інформації про гру якщо існує
    let game = rooms.get(&id).ok_or(StatusCode::NOT_FOUND)?.game.as_ref().ok_or(StatusCode::NO_CONTENT)?.read().await.clone(); 
    Ok(Response::builder().body(serde_json::to_string(&game).unwrap()))
}

#[handler]
pub async fn start( //функція для створення нової гри у кімнаті  
    Path(id): Path<String>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    rooms_ptr: Data<&Arc<RwLock<Rooms>>>,
) -> Result<StatusCode, StatusCode> {
    let (_players, mut rooms, player) =
        prelude(state.as_ref(), user, players_ptr.deref(), rooms_ptr.deref()).await?;
    rooms.update(&id, |room| {
        if player.uuid() == room.owner() {
            executor::block_on(room.start_game()).map_err(|_| StatusCode::CONFLICT) //блокування потоку доки не виконається асинхронний метод створення нової гри
        } else {
            Err(StatusCode::FORBIDDEN)
        }?;
        Ok::<(), StatusCode>(())
    })?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct Play { //структура, яка описує необхідні значення в шляху
    id: String, //ідентифікатор кімнати
    card_id: Option<usize>, //індекс карти у векторі карт гравця
}

#[handler]
pub async fn play(
    Path(Play { id, card_id }): Path<Play>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
    players_ptr: Data<&Arc<RwLock<crate::Players>>>,
    rooms_ptr: Data<&Arc<RwLock<Rooms>>>,
) -> Result<StatusCode, StatusCode> {
    let (_players, mut rooms, mut player) =
        prelude(&state.clone(), user, players_ptr.deref(), rooms_ptr.deref()).await?;
    let room = rooms.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    let state = state.as_ref();
    match room.play_game(*player.uuid(), card_id).await.map_err(|_e| { StatusCode::PRECONDITION_FAILED } )? { //виклик методу кімнати для ходіння у грі та обробка результату
        Ok::GameOver(players) => { //якщо гра завершилася
            for (index, player) in players.iter().enumerate() { //ітерація через усих гравців гри
                let mut savefile = state.savefiles.load(*player.id()).await.expect("err");

                savefile.games_played += 1; //збільшення кількості зіграних ігор на 1
                if index < players.len() / 2 { //вважати виграшом якщо вибув з гри раніше за половину гравців
                    savefile.wins += 1; //збільшення кількості виграшів    
                } else {
                    savefile.loses += 1; //збільшення кількості програшів
                }
                savefile.cards_had += *player.cards_had() as u64; //збільшення кількості карт які гравець тримав загалом
                savefile.points += *player.points(); //збільшення кількості очків
                savefile.max_points = savefile.max_points.max(*player.points() as u16); //порівняння значення найбільшої кількості очок за гру та їх назначення
            
                state.savefiles.save(savefile.into()).await;
            }
        },
        _ => {},
    }
    Ok(StatusCode::OK)
}