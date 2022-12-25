use std::{fmt::Display, io::stdin, str::FromStr};

use spotify_oauth::{SpotifyAuth, SpotifyCallback, SpotifyScope, SpotifyToken};

/// # Panics
/// Panics if the environment variables are not set
#[must_use]
pub fn get_auth() -> SpotifyAuth {
    dotenv::dotenv().ok();
    let client_id = match std::env::var("SPOTIFY_CLIENT_ID") {
        Ok(id) => id,
        Err(e) => panic!("Error getting client_id: {e}"),
    };

    let client_secret = match std::env::var("SPOTIFY_CLIENT_SECRET") {
        Ok(secret) => secret,
        Err(e) => panic!("Error getting client_secret: {e}"),
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

// # Panics
pub async fn get_token(auth: &SpotifyAuth) -> SpotifyToken {
    let auth_url = match auth.authorize_url() {
        Ok(url) => url,
        Err(e) => panic!("Error with url: {e}"),
    };
    match open::that(auth_url) {
        Ok(_) => println!("Opened url in browser. Please login and copy the url from the browser"),
        Err(e) => panic!("Error: {e}"),
    }

    let token = parse_token_res();

    match token
        .convert_into_token(
            auth.client_id.clone(),
            auth.client_secret.clone(),
            auth.redirect_uri.clone(),
        )
        .await
    {
        Ok(token) => token,
        Err(e) => panic!("Error converting into token: {e}"),
    }
}

fn get_buffer() -> String {
    let mut buffer = String::new();
    loop {
        match stdin().read_line(&mut buffer) {
            Ok(_) => return buffer,
            Err(e) => println!("Error reading line: {e}, please try again"),
        }
    }
}

fn parse_token_res() -> SpotifyCallback {
    loop {
        let buffer = get_buffer();
        match SpotifyCallback::from_str(buffer.trim()) {
            Ok(token) => return token,
            Err(e) => println!("Error parsing token: {e}, please try again"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub name: String,
    pub artist: String,
}

impl Song {
    #[must_use]
    pub const fn new(name: String, artist: String) -> Self {
        Self { name, artist }
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {}", self.name, self.artist)
    }
}
