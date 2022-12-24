use spotify_music_vid::Song;

struct YoutubeClient {
    client: reqwest::Client,
}

impl YoutubeClient {
    pub async fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_song_vid(&self, song: Song) -> String {
        let query = format!("{} {} music video", song.artist, song.name);
        let res = self
            .client
            .get("https://www.googleapis.com/youtube/v3/search")
            .query(&[("part", "snippet"), ("q", &query), ("type", "video")])
            .send()
            .await
            .unwrap();
        let res = res.text().await.unwrap();
        println!("{}", res);
        return res;
    }
}
