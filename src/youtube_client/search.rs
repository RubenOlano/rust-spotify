use serde::{Deserialize, Serialize};
use spotify_music_vid::Song;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub(crate) kind: String,
    pub(crate) etag: String,
    #[serde(rename = "nextPageToken")]
    pub(crate) next_page_token: String,
    #[serde(rename = "regionCode")]
    pub(crate) region_code: String,
    #[serde(rename = "pageInfo")]
    pub(crate) page_info: PageInfo,
    pub(crate) items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub(crate) kind: String,
    pub(crate) etag: String,
    pub(crate) id: Id,
    pub(crate) snippet: Snippet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Id {
    pub(crate) kind: String,
    #[serde(rename = "videoId")]
    pub(crate) video_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snippet {
    #[serde(rename = "publishedAt")]
    pub(crate) published_at: String,
    #[serde(rename = "channelId")]
    pub(crate) channel_id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) thumbnails: Thumbnails,
    #[serde(rename = "channelTitle")]
    pub(crate) channel_title: String,
    #[serde(rename = "liveBroadcastContent")]
    pub(crate) live_broadcast_content: String,
    #[serde(rename = "publishTime")]
    pub(crate) publish_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnails {
    #[serde(rename = "default")]
    pub(crate) thumbnails_default: Default,
    pub(crate) medium: Default,
    pub(crate) high: Default,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Default {
    pub(crate) url: String,
    pub(crate) width: i64,
    pub(crate) height: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "totalResults")]
    pub(crate) total_results: i64,
    #[serde(rename = "resultsPerPage")]
    pub(crate) results_per_page: i64,
}

impl ListResponse {
    fn get_video_id(&self) -> String {
        if let Some(video_id) = &self.items[0].id.video_id {
            return video_id.clone();
        }
        tracing::error!("No video id found");
        "CJtvnepMVAU".to_string()
    }

    pub fn get_vid_url(&self, song: &Song) -> String {
        let video_id = self.get_video_id();
        let duration = song.progress;
        format!("https://www.youtube.com/watch?v={video_id}&t={duration}")
    }
}
