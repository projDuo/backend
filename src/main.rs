mod infrastructure;
mod domain;
mod application;

use poem::{
    middleware::{ AddData, Cors }, EndpointExt,
    listener::TcpListener, Server,
};
use chrono::TimeDelta;
use std::sync::Arc;

use seaorm_migration::{Migrator, MigratorTrait};
use infrastructure::http::*;

use infrastructure::postgres::Postgres;
use infrastructure::memory::room::RoomStorage;
use infrastructure::memory::channel::ChannelStorage;

use infrastructure::eventbus::room::RoomEventBus;
use infrastructure::eventbus::room::RoomIEventBus;
use infrastructure::eventbus::game::*;
use infrastructure::eventbus::activity::ActivityEventBus;
use infrastructure::eventbus::activity::ActivityIEventBus;
use infrastructure::eventbus::chat::ChatEventBus;

type AccountsService = application::Accounts<Postgres>;
type TokenProvider = application::Jwt;
type AuthService = application::Auth<
        AccountsService,
        Postgres,
        TokenProvider>;
type SavefilesService = Arc<dyn domain::savefiles::SavefilesService + Send + Sync>;
type GameHistoryService = Arc<dyn domain::game_history::GameHistoryService + Send + Sync>;
type RoomService = application::Room;
type GameService = application::Game;
type ActivityService = application::Activity;
type ChatService = application::Chat;
type MutedService = Arc<dyn domain::muted::MutedService + Send + Sync>;

struct AppState {
    accounts: Arc<AccountsService>,
    auth: AuthService,
    savefiles: SavefilesService,
    rooms: Arc<RoomService>,
    games: Arc<GameService>,
    game_history: GameHistoryService,
    activities: Arc<ActivityService>,
    chats: Arc<ChatService>,
    muted: MutedService,
    room_event_bus: Arc<RoomEventBus>,
    game_event_bus: Arc<GameEventBus>,
    activity_event_bus: Arc<ActivityEventBus>,
    chat_event_bus: Arc<ChatEventBus>,
}

impl AppState {
    pub fn new(db: Postgres) -> anyhow::Result<Self> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET is missing"))?;
            
        let refresh_expires_after = std::env::var("JWT_REFRESH_EXPIRES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .map(TimeDelta::days)
            .unwrap_or_else(|| TimeDelta::days(30));

        let access_expires_after = std::env::var("JWT_ACCESS_EXPIRES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .map(TimeDelta::minutes)
            .unwrap_or_else(|| TimeDelta::minutes(15));

        let room_storage = Arc::new(RoomStorage::new(Arc::new(db.clone())));

        let room_event_bus = Arc::new(RoomEventBus::new(12));
        let room_ievent_bus = Arc::new(RoomIEventBus::new(12));
        let game_event_bus = Arc::new(GameEventBus::new(12));
        let game_ievent_bus = Arc::new(GameIEventBus::new(12));
        let activity_event_bus = Arc::new(ActivityEventBus::new(12));
        let activity_ievent_bus = Arc::new(ActivityIEventBus::new(12));
        let chat_event_bus = Arc::new(ChatEventBus::new(12));
        let channel_storage = ChannelStorage::new().into_arc();

        let accounts = Arc::new(AccountsService::new(db.clone()));
        let jwt = TokenProvider::new(secret, refresh_expires_after, access_expires_after);
        let auth = AuthService::new(accounts.clone(), db.clone(), jwt);
        let savefiles: SavefilesService =
            Arc::new(application::Savefiles::new(db.clone()));
        let game_history: GameHistoryService =
            Arc::new(application::GameHistory::new(db.clone()));
        let games = GameService::new(
            game_event_bus.clone(),
            game_ievent_bus.clone(),
            savefiles.clone(),
            game_history.clone(),
            activity_ievent_bus.clone(),
        );
        let activities = ActivityService::new(
            application::activity::ACTIVITY_HEARTBEAT_SECS,
            room_ievent_bus.clone(),
            game_ievent_bus.clone(),
            activity_event_bus.clone(),
            activity_ievent_bus.clone(),
        );
        let muted_repo: Arc<dyn domain::muted::MutedRepository + Send + Sync> = Arc::new(db.clone());
        let muted: MutedService = Arc::new(application::Muted::new(muted_repo.clone()));
        let chats = application::Chat::new(
            Arc::new(db.clone()),
            chat_event_bus.clone(),
            channel_storage,
            muted_repo.clone(),
        );
        let rooms = RoomService::new(
            room_storage.clone(),
            room_storage,
            room_event_bus.clone(),
            chats.clone(),
            chat_event_bus.clone(),
            room_ievent_bus.clone(),
            game_ievent_bus.clone(),
            activity_ievent_bus.clone(),
            activities.clone(),
        );

        Ok(Self {
            accounts,
            auth,
            savefiles,
            rooms,
            games,
            game_history,
            activities,
            chats,
            muted,
            room_event_bus: room_event_bus.clone(),
            game_event_bus: game_event_bus.clone(),
            activity_event_bus: activity_event_bus.clone(),
            chat_event_bus: chat_event_bus.clone(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let db_host = std::env::var("DB_HOST").expect("DB_HOST is missing");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME is missing");
    let db_port = std::env::var("DB_PORT").expect("DB_PORT is missing")
        .parse::<u16>().expect("DB_PORT is not a valid number");
    let db_user = std::env::var("DB_USER").expect("DB_USER is missing");
    let db_pass = std::env::var("DB_PASS").expect("DB_PASS is missing");

    let pg_details = infrastructure::postgres::PostgresDetails::new(
        &db_host,
        &db_name,
        db_port,
        &db_user,
        &db_pass,
    );

    let db = infrastructure::postgres::Postgres::from_details(pg_details).await
        .expect("Failed to connect to the database");
    
    Migrator::up(db.connection(), None)
        .await
        .expect("Failed to run database migrations");

    let state = AppState::new(db.clone())?;
    
    let app = api_routes(Arc::new(state.auth.clone()), Arc::new(db.clone()))
        .with(Cors::new().allow_origin_regex("*")) //Налаштування CORS політики
        .with(AddData::new(Arc::new(db.clone()))) //Передача посилання на з'єднання БД в аргументи функцій
        .with(AddData::new(Arc::new(state)));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("PORT is not a valid number");

    let bind_addr = format!("0.0.0.0:{}", port);
    println!("Starting server on http://{}", bind_addr);

    Server::new(TcpListener::bind(bind_addr))
        .run(app)
        .await?;

    Ok(())
}
