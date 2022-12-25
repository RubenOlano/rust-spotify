use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use spotify_music_vid::Song;

impl PlaybackState {
    pub fn is_diff(&self, other: &PlaybackState) -> bool {
        self.item.id != other.item.id
    }

    pub fn get_currently_playing(&self) -> Song {
        let item = &self.item;
        return Song::new(item.name.clone(), item.artists[0].name.clone());
    }
}

impl Display for PlaybackState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item = &self.item;
        write!(
            f,
            "Currently playing: {} by {}",
            item.name, item.artists[0].name
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybackState {
    pub(crate) device: Device,
    pub(crate) shuffle_state: bool,
    pub(crate) repeat_state: String,
    pub(crate) timestamp: i64,
    pub(crate) context: Context,
    pub(crate) progress_ms: i64,
    pub(crate) item: Item,
    pub(crate) currently_playing_type: String,
    pub(crate) actions: Actions,
    pub(crate) is_playing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Actions {
    pub(crate) disallows: Disallows,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Disallows {
    pub(crate) resuming: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub(crate) external_urls: ExternalUrls,
    pub(crate) href: String,
    #[serde(rename = "type")]
    pub(crate) context_type: String,
    pub(crate) uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalUrls {
    pub(crate) spotify: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub(crate) id: String,
    pub(crate) is_active: bool,
    pub(crate) is_private_session: bool,
    pub(crate) is_restricted: bool,
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) device_type: String,
    pub(crate) volume_percent: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub(crate) album: Album,
    pub(crate) artists: Vec<Artist>,
    pub(crate) available_markets: Vec<String>,
    pub(crate) disc_number: i64,
    pub(crate) duration_ms: i64,
    pub(crate) explicit: bool,
    pub(crate) external_ids: ExternalIds,
    pub(crate) external_urls: ExternalUrls,
    pub(crate) href: String,
    pub(crate) id: String,
    pub(crate) is_local: bool,
    pub(crate) name: String,
    pub(crate) popularity: i64,
    pub(crate) preview_url: String,
    pub(crate) track_number: i64,
    #[serde(rename = "type")]
    pub(crate) item_type: String,
    pub(crate) uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    pub(crate) album_type: String,
    pub(crate) artists: Vec<Artist>,
    pub(crate) available_markets: Vec<String>,
    pub(crate) external_urls: ExternalUrls,
    pub(crate) href: String,
    pub(crate) id: String,
    pub(crate) images: Vec<Image>,
    pub(crate) name: String,
    pub(crate) release_date: String,
    pub(crate) release_date_precision: String,
    pub(crate) total_tracks: i64,
    #[serde(rename = "type")]
    pub(crate) purple_type: String,
    pub(crate) uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub(crate) external_urls: ExternalUrls,
    pub(crate) href: String,
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) artist_type: String,
    pub(crate) uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub(crate) height: i64,
    pub(crate) url: String,
    pub(crate) width: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalIds {
    pub(crate) isrc: String,
}
