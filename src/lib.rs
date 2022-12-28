use color_eyre::Result;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rspotify::{
    model::{CurrentlyPlayingContext, PlayableItem},
    prelude::{BaseClient, OAuthClient},
    scopes, AuthCodeSpotify, Credentials, OAuth,
};
use std::{fmt::Display, time::Duration};
use tracing::info;
use warp::ws::{Message, WebSocket};

type Reader = SplitStream<WebSocket>;
type Writer = SplitSink<WebSocket, Message>;

/// # Panics
/// Panics if the environment variables are not set
/// requires `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET`
/// # Errors
/// Returns an error if the environment variables are not set
pub fn get_auth() -> Result<AuthCodeSpotify> {
    info!("Getting env variables");
    dotenv::dotenv().ok();
    let client_id = std::env::var("SPOTIFY_CLIENT_ID")?;
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")?;

    let creds = Credentials::new(client_id.as_str(), client_secret.as_str());

    let mut oauth = OAuth::default();
    oauth.redirect_uri = "http://localhost:8000/callback".to_string();

    oauth.scopes = scopes!["user-read-currently-playing", "user-read-playback-state"];
    let spotify = AuthCodeSpotify::new(creds, oauth);
    Ok(spotify)
}

/// # Panics
/// Panics if the environment variables are not set
/// requires `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET`
/// # Errors
/// Returns an error if the environment variables are not set
pub async fn get_token(
    auth: &AuthCodeSpotify,
    read: &mut Reader,
    write: &mut Writer,
) -> Result<()> {
    let auth_url = auth.get_authorize_url(true)?;

    info!("Sending auth url to client");
    let msg = Message::text(auth_url.as_str());
    write.send(msg).await?;
    info!("Sent auth url to client");
    // wait for the client to send the code back
    let url = read.next().await;
    if let Some(Ok(url)) = url {
        let url = url.to_str().unwrap();
        info!("Got code from client");
        let code = auth.parse_response_code(url).unwrap();
        auth.request_token(code.as_str()).await?;
        auth.write_token_cache().await?;
    } else {
        return Err(color_eyre::eyre::eyre!("No url from client"));
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Song {
    pub name: String,
    pub artist: String,
    pub progress: u32,
}

impl Song {
    pub const fn new(name: String, artist: String, progress: u32) -> Self {
        Self {
            name,
            artist,
            progress,
        }
    }

    pub fn from_context(ctx: CurrentlyPlayingContext) -> Result<Self> {
        let item = match ctx.item {
            Some(item) => item,
            None => return Err(color_eyre::eyre::eyre!("No item in context")),
        };
        if let PlayableItem::Track(track) = item {
            let artist = track.artists[0].name.clone();
            let name = track.name.clone();
            let progress = ctx.progress.unwrap_or(Duration::default());
            let progress = progress.as_secs() as u32;
            Ok(Self::new(name, artist, progress))
        } else {
            Err(color_eyre::eyre::eyre!("No track in context"))
        }
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {}", self.name, self.artist)
    }
}
