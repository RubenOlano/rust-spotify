mod spotify_client;
mod youtube_client;

use color_eyre::eyre::Result;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run_program().await {
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

async fn run_program() -> Result<()> {
    init()?;
    info!("Start unwrapping auth");
    let auth = get_auth()?;
    get_token(&auth).await?;

    let mut client = SpotifyClient::new(auth)?;
    client.start_polling().await?;
    Ok(())
}
