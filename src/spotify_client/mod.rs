use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION, CONTENT_TYPE};
use spotify_oauth::{SpotifyAuth, SpotifyToken};
use tokio::time::{sleep, Duration};
use url::Url;

use crate::{
    player_state::PlaybackState,
    youtube_client::{self, YoutubeClient},
};

#[derive(Debug)]
pub enum ClientError {
    CreatedError(String),
    ReqwestError(reqwest::Error),
    NoStateError(String),
    YoutubeError(youtube_client::ClientError),
    HeaderError(InvalidHeaderValue),
}

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
    pub async fn new(auth: SpotifyAuth, token: SpotifyToken) -> Result<Self, ClientError> {
        let env_vars = EnvVars::load_vars();
        Ok(Self {
            token,
            client_id: auth.client_id,
            client_secret: auth.client_secret,
            callback_url: Url::parse(&env_vars.callback_url)
                .map_err(|e| ClientError::CreatedError(e.to_string()))?,
            prev_state: None,
            yt_client: YoutubeClient::new()
                .await
                .map_err(ClientError::YoutubeError)?,
        })
    }

    async fn get_state_loop(&mut self) -> Result<PlaybackState, ClientError> {
        let mut state = self.get_state().await;
        while let Err(e) = state {
            println!("Error getting state: {e:?}");
            println!("Retrying in 5 seconds");
            sleep(Duration::from_secs(5)).await;
            state = self.get_state().await;
        }
        state
    }

    async fn get_state(&self) -> Result<PlaybackState, ClientError> {
        let client = reqwest::Client::new();
        let headers = self.get_headers()?;
        let res = client
            .get("https://api.spotify.com/v1/me/player")
            .headers(headers)
            .send()
            .await
            .map_err(ClientError::ReqwestError)?;

        if res.status() == 204 {
            return Err(ClientError::NoStateError("No state available".to_string()));
        }

        let state = res.json::<PlaybackState>().await;
        match state {
            Ok(state) => Ok(state),
            Err(e) => Err(ClientError::ReqwestError(e)),
        }
    }

    fn get_headers(&self) -> Result<HeaderMap, ClientError> {
        let token_string: HeaderValue = format!("Bearer {}", self.token.access_token)
            .parse()
            .map_err(ClientError::HeaderError)?;

        let json: HeaderValue = "application/json"
            .parse()
            .map_err(ClientError::HeaderError)?;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token_string);
        headers.insert(CONTENT_TYPE, json);
        Ok(headers)
    }

    pub async fn start_polling(&mut self) -> Result<(), ClientError> {
        while let Ok(state) = self.get_state_loop().await {
            if self.check_state_change(&state) {
                self.handle_state_change(state).await;
            }

            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    async fn handle_state_change(&mut self, state: PlaybackState) {
        let song = state.get_currently_playing();
        let vid = self.yt_client.get_song_vid(song).await;
        let Ok(vid) = vid else {
            return;
        };
        match open::that(vid) {
            Ok(_) => println!("Opened {}", state.get_currently_playing()),
            Err(e) => println!("Error opening video: {e:?}"),
        }
    }

    fn check_state_change(&mut self, state: &PlaybackState) -> bool {
        if let Some(prev_state) = &self.prev_state {
            if prev_state.is_diff(state) {
                self.update_state(state);
                return true;
            }
        }
        self.update_state(state);
        false
    }

    fn update_state(&mut self, state: &PlaybackState) {
        self.prev_state = Some(state.clone());
    }
}

impl EnvVars {
    fn load_vars() -> Self {
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
