mod player_state;
mod spotify_client;
mod youtube_client;

use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};

#[tokio::main]
async fn main() {
    let auth = get_auth();
    let token = get_token(&auth).await;

    let mut client = match SpotifyClient::new(auth, token).await {
        Ok(client) => client,
        Err(e) => {
            println!("Error creating client: {e:?}");
            return;
        }
    };
    match client.start_polling().await {
        Ok(_) => println!("Done"),
        Err(e) => println!("Error: {e:?}"),
    }
}
