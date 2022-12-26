mod spotify_client;
mod youtube_client;

use color_eyre::Result;
use futures_util::{stream::SplitSink, StreamExt};
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Message, Result as WsResult},
    WebSocketStream,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

type WsStream = WebSocketStream<TcpStream>;

#[tokio::main]
async fn main() -> WsResult<()> {
    // create websocket client
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;

    // only need to accept one connection
    let (stream, _) = listener.accept().await?;
    let peer = stream.peer_addr()?;
    info!("New connection from {}", peer);

    let ws_stream = accept_async(stream).await?;
    // get the write half of the websocket stream
    let (write, _) = ws_stream.split();

    // the write will be used to send messages to the client
    // mainly to send the youtube link to the client

    if let Err(e) = run_program(write).await {
        error!("Error: {}", e);
        println!("Error encountered during execution, see logs for more info");
    }
    Ok(())
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

async fn run_program(write: SplitSink<WsStream, Message>) -> Result<()> {
    init()?;
    info!("Start unwrapping auth");
    let auth = get_auth()?;
    get_token(&auth).await?;

    let mut client = SpotifyClient::new(auth, write)?;
    client.start_polling().await?;
    Ok(())
}
