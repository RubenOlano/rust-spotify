use std::{io::stdin, str::FromStr};

use spotify_oauth::{SpotifyAuth, SpotifyCallback, SpotifyScope, SpotifyToken};

pub async fn get_auth() -> SpotifyAuth {
    dotenv::dotenv().ok();
    let client_id = match std::env::var("SPOTIFY_CLIENT_ID") {
        Ok(id) => id,
        Err(e) => panic!("Error getting client_id: {}", e),
    };

    let client_secret = match std::env::var("SPOTIFY_CLIENT_SECRET") {
        Ok(secret) => secret,
        Err(e) => panic!("Error getting client_secret: {}", e),
    };

    SpotifyAuth::new(
        client_id,
        client_secret,
        "code".to_string(),
        "http://localhost:8000/callback".to_string(),
        vec![
            SpotifyScope::UserReadPlaybackState,
            SpotifyScope::UserReadCurrentlyPlaying,
        ],
        true,
    )
}

pub async fn get_token(auth: &SpotifyAuth) -> SpotifyToken {
    let auth_url = match auth.authorize_url() {
        Ok(url) => url,
        Err(e) => panic!("Error with url: {}", e),
    };
    match open::that(auth_url) {
        Ok(_) => println!("Opened url in browser. Please login and copy the url from the browser"),
        Err(e) => panic!("Error: {}", e),
    }

    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => println!("Successfully read url from stdin"),
        Err(e) => panic!("Error: {}", e),
    }

    let token = match SpotifyCallback::from_str(buffer.trim()) {
        Ok(token) => token,
        Err(e) => panic!("Error getting token: {}", e),
    };
    match token
        .convert_into_token(
            auth.client_id.clone(),
            auth.client_secret.clone(),
            auth.redirect_uri.clone(),
        )
        .await
    {
        Ok(token) => token,
        Err(e) => panic!("Error converting into token: {}", e),
    }
}
