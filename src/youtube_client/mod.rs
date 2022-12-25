mod search;

use color_eyre::eyre::Result;
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Client,
};
use spotify_music_vid::Song;
use tracing::info;

use self::search::ListResponse;

#[derive(Debug, Clone)]
pub struct YoutubeClient {
    client: Client,
    api_key: String,
}

impl YoutubeClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            api_key: get_env_var()?,
        })
    }

    pub async fn get_song_vid(&self, song: &Song) -> Result<String> {
        let query = format!("{} {} music video", song.artist, song.name);
        let res = self.send_req(&query, song).await?;
        Ok(res)
    }

    async fn send_req(&self, query: &str, song: &Song) -> Result<String> {
        let headers = get_headers()?;

        let res = self
            .client
            .get("https://youtube.googleapis.com/youtube/v3/search")
            .headers(headers)
            .query(&[
                ("part", "snippet"),
                ("q", query),
                ("key", self.api_key.as_str()),
            ])
            .send()
            .await?;

        let res = self.parse_res(res, song).await?;
        Ok(res)
    }

    async fn parse_res(&self, res: reqwest::Response, song: &Song) -> Result<String> {
        let res: ListResponse = res.json().await?;
        Ok(res.get_vid_url(song))
    }
}

fn get_env_var() -> Result<String> {
    info!("Loading environment variables");
    dotenv::dotenv()?;
    let api_key = std::env::var("YOUTUBE_API_KEY")?;
    Ok(api_key)
}

fn get_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let json = "application/json".parse()?;
    headers.insert(ACCEPT, json);
    Ok(headers)
}
