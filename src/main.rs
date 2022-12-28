mod spotify_client;
mod youtube_client;

use color_eyre::Result;
use futures_util::{stream::SplitSink, StreamExt};
use rspotify::AuthCodeSpotify;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
type Writer = SplitSink<WebSocket, Message>;

#[tokio::main]
async fn main() {
    init().unwrap();
    // create websocket client
    let routes = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|socket| async move {
            let (mut tx, mut rx) = socket.split();
            let auth = get_auth().unwrap();
            get_token(&auth, &mut rx, &mut tx).await.unwrap();
            run_program(tx, auth).await.unwrap();
        })
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

async fn run_program(write: Writer, auth: AuthCodeSpotify) -> Result<()> {
    let mut client = SpotifyClient::new(auth, write)?;
    client.start_polling().await?;
    Ok(())
}
