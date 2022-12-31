mod lru;
mod spotify_client;
mod youtube_client;

use std::sync::Arc;

use color_eyre::Result;
use futures_util::{lock::Mutex, stream::SplitSink, StreamExt};
use lru::LRU;
use rspotify::AuthCodeSpotify;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token, Song};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
type Writer = SplitSink<WebSocket, Message>;
type Cache = Arc<Mutex<LRU<Song, String>>>;

#[tokio::main]
async fn main() {
    init().unwrap();
    let lru: LRU<Song, String> = LRU::new(100);
    let cache: Cache = Arc::new(Mutex::new(lru));
    // create websocket client
    let routes = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let cache = Arc::clone(&cache);
            ws.on_upgrade(move |socket| handle_connect(socket, cache))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn init() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn run_program(write: Writer, auth: AuthCodeSpotify, cache: Cache) -> Result<()> {
    let mut client = SpotifyClient::new(auth, write, cache)?;
    client.start_polling().await?;
    Ok(())
}

async fn handle_connect(socket: WebSocket, cache: Cache) {
    let (mut tx, mut rx) = socket.split();
    let auth = get_auth().unwrap();
    get_token(&auth, &mut rx, &mut tx).await.unwrap();
    run_program(tx, auth, cache).await.unwrap();
}
