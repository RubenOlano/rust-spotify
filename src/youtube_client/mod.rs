struct YoutubeClient {
    client: reqwest::Client,
}

impl YoutubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_video_id(&self, query: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&key={}",
            query, self.api_key
        );
        let res = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let video_id = res["items"][0]["id"]["videoId"].as_str().unwrap();
        Ok(video_id.to_string())
    }
}
