use dotenv::dotenv;
use spotify_oauth::{SpotifyAuth, SpotifyToken};
use url::Url;

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
}

impl SpotifyClient {
    pub async fn new(auth: SpotifyAuth, token: SpotifyToken) -> Self {
        let env_vars = EnvVars::load_vars();
        return SpotifyClient {
            token,
            client_id: auth.client_id,
            client_secret: auth.client_secret,
            callback_url: Url::parse(&env_vars.callback_url).unwrap(),
        };
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
            Err(_) => {
                println!("No callback url found. Using default");
                "http://localhost:8000/callback".to_string()
            }
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
        }
    }
}
