use std::{fmt::Display, io::stdin, str::FromStr};

use color_eyre::Result;
use spotify_oauth::{SpotifyAuth, SpotifyCallback, SpotifyScope, SpotifyToken};
use tracing::*;

/// # Panics
/// Panics if the environment variables are not set
#[must_use]
pub fn get_auth() -> Result<SpotifyAuth> {
    info!("Getting env variables");
    dotenv::dotenv().ok();
    let client_id = std::env::var("SPOTIFY_CLIENT_ID")?;

    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")?;

    Ok(SpotifyAuth::new(
        client_id,
        client_secret,
        "code".to_string(),
        "http://localhost:8000/callback".to_string(),
        vec![
            SpotifyScope::UserReadPlaybackState,
            SpotifyScope::UserReadCurrentlyPlaying,
        ],
        true,
    ))
}

/// # Panics
/// Panics if the environment variables are not set
pub async fn get_token(auth: &SpotifyAuth) -> Result<SpotifyToken> {
    let auth_url = auth.authorize_url()?;

    info!("Opening browser to {auth_url}");
    open::that(auth_url)?;

    let token = parse_token_res();

    info!("Converting into token");
    Ok(token
        .convert_into_token(
            auth.client_id.clone(),
            auth.client_secret.clone(),
            auth.redirect_uri.clone(),
        )
        .await?)
}

fn get_buffer() -> String {
    info!("Getting buffer");
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
