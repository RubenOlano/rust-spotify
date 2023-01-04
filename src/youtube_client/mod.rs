mod search;

use color_eyre::eyre::Result;
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Client, Response,
};
use spotify_music_vid::Song;
use tracing::{error, info, instrument};

use self::search::ListResponse;

#[derive(Debug, Clone)]
pub struct YoutubeClient {
    client: Client,
    api_key: String,
}

impl YoutubeClient {
    /// Creates a new [`YoutubeClient`].
    /// # Errors
    /// This function will return an error if the environment variable `YOUTUBE_API_KEY` is not set.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            api_key: get_env_var()?,
        })
    }

    /// Gets the video url for a song given [`Song`].
    /// This function will search for the song on youtube and return the first result.
    /// # Errors
    /// This function will return an error if the request fails or if the response is not valid.
    #[instrument(skip(self))]
    pub async fn get_song_vid(&self, song: &Song) -> Result<(String, String)> {
        let query = format!("{} {} music video", song.artist, song.name);
        let res = self.send_req(&query).await?;
        let res = self.parse_res(res, song).await?;
        Ok(res)
    }

    /// Sends the request to youtube.
    /// # Errors
    /// This function will return an error if the request fails.
    async fn send_req(&self, query: &str) -> Result<Response> {
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
            .await;

        Ok(res?)
    }

    /// Parses the response from youtube.
    /// # Errors
    /// This function will return an error if the response is not valid.
    async fn parse_res(&self, res: Response, song: &Song) -> Result<(String, String)> {
        let res: ListResponse = res.json().await?;
        Ok(res.get_vid_url(song))
    }
}

impl Default for YoutubeClient {
    fn default() -> Self {
        match Self::new() {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to create YoutubeClient: {e}");
                panic!("Failed to create YoutubeClient: {e}");
            }
        }
    }
}

fn get_env_var() -> Result<String> {
    info!("Loading environment variables");
    // dotenv::dotenv()?;
    let api_key = std::env::var("YOUTUBE_API_KEY")?;
    Ok(api_key)
}

fn get_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let json = "application/json".parse()?;
    headers.insert(ACCEPT, json);
    Ok(headers)
}
