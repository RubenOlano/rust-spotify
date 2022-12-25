use color_eyre::eyre::{Error, Result};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use spotify_oauth::{SpotifyAuth, SpotifyToken};
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use url::Url;

use crate::{player_state::PlaybackState, youtube_client::YoutubeClient};

#[derive(Debug, Clone)]
struct EnvVars {
    callback_url: String,
}

#[derive(Debug)]
pub struct SpotifyClient {
    pub token: SpotifyToken,
    pub client_id: String,
    pub client_secret: String,
    callback_url: Url,
    prev_state: Option<PlaybackState>,
    yt_client: YoutubeClient,
}

impl SpotifyClient {
    /// # Panics
    ///
    /// Panics if the environment variables are not set
    pub fn new(auth: SpotifyAuth, token: SpotifyToken) -> Result<Self> {
        info!("Creating new SpotifyClient and loading environment variables");
        let env_vars = EnvVars::load_vars();
        Ok(Self {
            token,
            client_id: auth.client_id,
            client_secret: auth.client_secret,
            callback_url: Url::parse(&env_vars.callback_url)?,
            prev_state: None,
            yt_client: YoutubeClient::new()?,
        })
    }

    async fn get_state_loop(&mut self) -> Result<PlaybackState> {
        let mut state = self.get_state().await;
        while let Err(ref e) = state {
            println!("Something went wrong, retrying in 5 seconds");
            error!("Failed to get state: {e}");
            sleep(Duration::from_secs(5)).await;
            state = self.get_state().await;
        }
        state
    }

    async fn get_state(&self) -> Result<PlaybackState> {
        let client = reqwest::Client::new();
        let headers = self.get_headers()?;
        let res = client
            .get("https://api.spotify.com/v1/me/player")
            .headers(headers)
            .send()
            .await?;

        if res.status() == 204 {
            info!("No state found, fetch again");
            return Err(Error::msg("No state"));
        }

        let state = res.json::<PlaybackState>().await?;
        Ok(state)
    }

    fn get_headers(&self) -> Result<HeaderMap> {
        info!("Getting headers");
        let token_string: HeaderValue = format!("Bearer {}", self.token.access_token).parse()?;

        let json: HeaderValue = "application/json".parse()?;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token_string);
        headers.insert(CONTENT_TYPE, json);
        Ok(headers)
    }

    pub async fn start_polling(&mut self) -> Result<()> {
        info!("Starting polling");
        while let Ok(state) = self.get_state_loop().await {
            if self.check_state_change(&state) {
                info!("State changed, opening video");
                self.handle_state_change(state).await?;
            }

            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    async fn handle_state_change(&mut self, state: PlaybackState) -> Result<()> {
        let song = state.get_currently_playing();
        let vid = self.yt_client.get_song_vid(song).await?;

        match open::that(vid) {
            Ok(_) => info!("Opened {}", state.get_currently_playing()),
            Err(e) => error!("Error opening video: {e:?}"),
        }
        Ok(())
    }

    fn check_state_change(&mut self, state: &PlaybackState) -> bool {
        if let Some(prev_state) = &self.prev_state {
            if prev_state.is_diff(state) {
                self.update_state(state);
                return true;
            }
            return false;
        }
        self.update_state(state);
        true
    }

    fn update_state(&mut self, state: &PlaybackState) {
        self.prev_state = Some(state.clone());
    }
}

impl EnvVars {
    fn load_vars() -> Self {
        info!("Loading environment variables");
        dotenv().ok();

        let callback_url = std::env::var("SPOTIFY_CALLBACK_URL")
            .map_or_else(|_| "http://localhost:8888/callback".to_string(), |v| v);

        Self { callback_url }
    }
}

impl Clone for SpotifyClient {
    fn clone(&self) -> Self {
        let clone_token = SpotifyToken {
            access_token: self.token.access_token.clone(),
            token_type: self.token.token_type.clone(),
            scope: self.token.scope.clone(),
            expires_in: self.token.expires_in,
            refresh_token: self.token.refresh_token.clone(),
            expires_at: self.token.expires_at,
        };
        Self {
            token: clone_token,
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            callback_url: self.callback_url.clone(),
            prev_state: None,
            yt_client: self.yt_client.clone(),
        }
    }
}
