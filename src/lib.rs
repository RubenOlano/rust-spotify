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
use std::fmt::Display;
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

    let oauth = OAuth {
        redirect_uri: "http://localhost:5173/callback".to_string(),
        scopes: scopes!["user-read-currently-playing", "user-read-playback-state"],
        ..Default::default()
    };
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
    let code = read.next().await;
    let code = code.ok_or(color_eyre::eyre::eyre!("No code from client"))??;
    info!("Got code from client, {:?}", code.clone());
    let code = handle_message(&code)?;
    info!("Got code from client");
    auth.request_token(&code).await?;
    auth.write_token_cache().await?;

    Ok(())
}

/// Handles a [`Message`] from the client.
/// returns the message as a string
/// # Errors
/// This function will return an error if the message received from the client is not a string
pub fn handle_message(msg: &Message) -> Result<String> {
    let msg = msg
        .to_str()
        .map_err(|_| color_eyre::eyre::eyre!("Could not convert message to string"))?;
    let msg = msg.trim();
    let msg = msg.to_lowercase();
    Ok(msg)
}

#[derive(Debug, Clone, sqlx::FromRow, sqlx::Decode)]
pub struct Song {
    pub name: String,
    pub artist: String,
    pub progress: i64,
}

impl Song {
    /// Creates a new [`Song`].
    #[must_use]
    pub const fn new(name: String, artist: String, progress: i64) -> Self {
        Self {
            name,
            artist,
            progress,
        }
    }

    /// Creates a new [`Song`] from a [`CurrentlyPlayingContext`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the [`CurrentlyPlayingContext`] does not contain a track.
    pub fn from_context(ctx: CurrentlyPlayingContext) -> Result<Self> {
        let item = ctx
            .item
            .ok_or(color_eyre::eyre::eyre!("No item in context"))?;
        if let PlayableItem::Track(track) = item {
            let artist = track.artists[0].name.clone();
            let name = track.name;
            let progress = ctx.progress.unwrap_or_default();
            let progress = progress.as_secs().try_into()?;
            Ok(Self::new(name, artist, progress))
        } else {
            Err(color_eyre::eyre::eyre!("No track in context"))
        }
    }

    #[must_use]
    pub fn get_embed_url(song_id: &str) -> String {
        format!("https://www.youtube.com/embed/{song_id}")
    }

    #[must_use]
    pub fn get_url_with_duration(song_id: &str, duration: &str) -> String {
        format!("https://www.youtube.com/embed/{song_id}?start={duration}&autoplay=1&")
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {}", self.name, self.artist)
    }
}
