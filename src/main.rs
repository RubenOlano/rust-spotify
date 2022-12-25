mod player_state;
mod spotify_client;
mod youtube_client;

use color_eyre::eyre::Result;
use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Error installing color_eyre");
    init().expect("Error initializing tracing");
    info!("Start unwrapping auth");
    let auth = get_auth().expect("Error getting auth");
    let token = get_token(&auth).await.expect("Error getting token");

    let mut client = match SpotifyClient::new(auth) {
        Ok(client) => client,
        Err(e) => {
            println!("Error creating client");
            error!("Error creating client: {e:?}");
            return;
        }
    };
    match client.start_polling().await {
        Ok(_) => println!("Done"),
        Err(e) => {
            println!("Something went wrong when polling");
            error!("Error polling: {e:?}");
        }
    }
}

fn init() -> Result<()> {
    dotenv::dotenv()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
