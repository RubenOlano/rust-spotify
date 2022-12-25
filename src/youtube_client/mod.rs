mod search;

use reqwest::{
    header::{HeaderMap, ACCEPT},
    Client,
};
use spotify_music_vid::Song;

use self::search::*;

#[derive(Debug)]
pub enum ClientError {
    ReqwestError(reqwest::Error),
    HeaderError(reqwest::header::InvalidHeaderValue),
    EnvVarError(std::env::VarError),
}

#[derive(Debug, Clone)]
pub struct YoutubeClient {
    client: Client,
    api_key: String,
}

impl YoutubeClient {
    pub async fn new() -> Result<Self, ClientError> {
        Ok(Self {
            client: reqwest::Client::new(),
            api_key: get_env_var()?,
        })
    }

    pub async fn get_song_vid(&self, song: Song) -> Result<String, ClientError> {
        let query = format!("{} {} music video", song.artist, song.name);

        let res = self.send_req(&query).await?;
        Ok(res)
    }

    async fn send_req(&self, query: &str) -> Result<String, ClientError> {
        let headers = get_headers()?;

        let res = self
            .client
            .get("https://youtube.googleapis.com/youtube/v3/search")
            .headers(headers)
            .query(&[
                ("part", "snippet"),
                ("q", &query),
                ("key", self.api_key.as_str()),
            ])
            .send()
            .await
            .map_err(ClientError::ReqwestError)?;

        let res = self.parse_res(res).await?;
        Ok(res)
    }

    async fn parse_res(&self, res: reqwest::Response) -> Result<String, ClientError> {
        let res: SearchListResponse = res.json().await.map_err(ClientError::ReqwestError)?;
        Ok(res.get_vid_url())
    }
}

fn get_env_var() -> Result<String, ClientError> {
    dotenv::dotenv().ok();
    let api_key = std::env::var("YOUTUBE_API_KEY").map_err(ClientError::EnvVarError)?;
    Ok(api_key)
}

fn get_headers() -> Result<HeaderMap, ClientError> {
    let mut headers = HeaderMap::new();
    let json = "application/json"
        .parse()
        .map_err(ClientError::HeaderError)?;
    headers.insert(ACCEPT, json);
    Ok(headers)
}