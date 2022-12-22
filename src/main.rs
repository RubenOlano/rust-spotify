mod spotify_client;

use spotify_client::SpotifyClient;
use spotify_music_vid::{get_auth, get_token};

#[tokio::main]
async fn main() {
    let auth = get_auth().await;
    let token = get_token(&auth).await;

    let client = SpotifyClient::new(auth, token).await;
    println!("{:#?}", client);
}
