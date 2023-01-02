mod db;
mod spotify_client;
mod youtube_client;

use std::sync::Arc;

use color_eyre::Result;
use db::config::Config;
use futures_util::{stream::SplitSink, StreamExt};
use rspotify::AuthCodeSpotify;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use sqlx::{Pool, Postgres};
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
type Writer = SplitSink<WebSocket, Message>;

#[tokio::main]
async fn main() -> Result<()> {
    init()?;
    let config = Config::from_env()?;
    let pool = config.create_db_pool().await?;
    let arc_pool = Arc::new(pool);

    // create websocket client
    let routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || Arc::clone(&arc_pool)))
        .map(|ws: warp::ws::Ws, pool_conn| {
            ws.on_upgrade(move |socket| handle_connect(socket, pool_conn))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    Ok(())
}

fn init() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn run_program(
    write: Writer,
    auth: AuthCodeSpotify,
    pool: Arc<Pool<Postgres>>,
) -> Result<()> {
    let mut client = SpotifyClient::new(auth, write, pool)?;
    client.start_polling().await?;
    Ok(())
}

async fn handle_connect(socket: WebSocket, pool: Arc<Pool<Postgres>>) {
    let (mut tx, mut rx) = socket.split();
    let auth = match get_auth() {
        Ok(auth) => auth,
        Err(e) => {
            error!("Failed to get auth: {e}");
            return;
        }
    };
    match get_token(&auth, &mut rx, &mut tx).await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to get token: {:?}", e);
            return;
        }
    };
    match run_program(tx, auth, pool).await {
        Ok(_) => (),
        Err(e) => error!("Failed to run program: {e}"),
    }
}
