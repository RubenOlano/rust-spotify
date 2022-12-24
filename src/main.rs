mod player_state;
mod spotify_client;
mod youtube_client;

use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};

#[tokio::main]
async fn main() {
    let auth = get_auth().await;
    let token = get_token(&auth).await;

    let mut client = SpotifyClient::new(auth, token).await;
    client.start_polling().await.unwrap();
}
