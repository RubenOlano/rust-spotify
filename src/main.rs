mod db;
mod spotify_client;
mod youtube_client;

use color_eyre::Result;
use db::config::Config;
use futures_util::{stream::SplitSink, StreamExt};
use rspotify::AuthCodeSpotify;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use sqlx::{Pool, Postgres};
use tracing::Level;
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

    // create websocket client
    let routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || pool.clone()))
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
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn run_program(write: Writer, auth: AuthCodeSpotify, pool: Pool<Postgres>) -> Result<()> {
    let mut client = SpotifyClient::new(auth, write, pool)?;
    client.start_polling().await?;
    Ok(())
}

async fn handle_connect(socket: WebSocket, pool: Pool<Postgres>) {
    let (mut tx, mut rx) = socket.split();
    let auth = get_auth().unwrap();
    get_token(&auth, &mut rx, &mut tx).await.unwrap();
    run_program(tx, auth, pool).await.unwrap();
}
