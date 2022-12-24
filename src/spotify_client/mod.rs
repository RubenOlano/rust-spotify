use dotenv::dotenv;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use spotify_oauth::{SpotifyAuth, SpotifyToken};
use tokio::time::{sleep, Duration};
use url::Url;

use crate::player_state::PlaybackState;

#[derive(Debug)]
pub enum ClientError {
    ReqwestError(reqwest::Error),
    NoStateError(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EnvVars {
    client_id: String,
    client_secret: String,
    callback_url: String,
}

#[derive(Debug)]
pub struct SpotifyClient {
    pub token: SpotifyToken,
    pub client_id: String,
    pub client_secret: String,
    callback_url: Url,
    prev_state: Option<PlaybackState>,
}

impl SpotifyClient {
    pub async fn new(auth: SpotifyAuth, token: SpotifyToken) -> Self {
        let env_vars = EnvVars::load_vars();
        return SpotifyClient {
            token,
            client_id: auth.client_id,
            client_secret: auth.client_secret,
            callback_url: Url::parse(&env_vars.callback_url).unwrap(),
            prev_state: None,
        };
    }

    async fn get_state_loop(&mut self) -> Result<PlaybackState, ClientError> {
        let mut state = self.get_state().await;
        while let Err(e) = state {
            println!("Error getting state: {:?}", e);
            println!("Retrying in 5 seconds");
            sleep(Duration::from_secs(5)).await;
            state = self.get_state().await;
        }
        state
    }

    async fn get_state(&self) -> Result<PlaybackState, ClientError> {
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.spotify.com/v1/me/player")
            .headers(self.get_headers())
            .send()
            .await;

        let res = match res {
            Ok(res) => res,
            Err(e) => return Err(ClientError::ReqwestError(e)),
        };
        if res.status() == 204 {
            return Err(ClientError::NoStateError("No state available".to_string()));
        }

        let state = res.json::<PlaybackState>().await;
        return match state {
            Ok(state) => Ok(state),
            Err(e) => Err(ClientError::ReqwestError(e)),
        };
    }

    fn get_headers(&self) -> HeaderMap {
        let token_string = format!("Bearer {}", self.token.access_token);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token_string.parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers
    }

    pub async fn start_polling(&mut self) -> Result<(), ClientError> {
        while let Ok(state) = self.get_state_loop().await {
            if self.check_state_change(&state) {
                println!("State changed!");
            }
            println!("Currently Playing: {}", state.progress_as_string());
            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    fn check_state_change(&mut self, state: &PlaybackState) -> bool {
        if let Some(prev_state) = &self.prev_state {
            if prev_state.is_diff(&state) {
                self.update_state(state);
                return true;
            }
        }
        self.update_state(state);
        false
    }

    fn update_state(&mut self, state: &PlaybackState) {
        let new_state = state.clone();
        self.prev_state = Some(new_state);
    }
}

impl EnvVars {
    fn load_vars() -> Self {
        dotenv().ok();

        let client_id = match std::env::var("SPOTIFY_CLIENT_ID") {
            Ok(id) => id,
            Err(e) => panic!("Error getting client_id: {}", e),
        };

        let client_secret = match std::env::var("SPOTIFY_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(e) => panic!("Error getting client_secret: {}", e),
        };

        let callback_url = match std::env::var("SPOTIFY_CALLBACK_URL") {
            Ok(url) => url,
            Err(_) => "http://localhost:8000/callback".to_string(),
        };

        return EnvVars {
            client_id,
            client_secret,
            callback_url,
        };
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
        SpotifyClient {
            token: clone_token,
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            callback_url: self.callback_url.clone(),
            prev_state: None,
        }
    }
}
