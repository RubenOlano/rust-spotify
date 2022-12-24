use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Device {
    id: String,
    is_active: bool,
    is_private_session: bool,
    is_restricted: bool,
    name: String,
    #[serde(rename(deserialize = "type"))]
    dev_type: String,
    volume_percent: i32,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct ExternalUrls {
    spotify: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Artist {
    external_urls: ExternalUrls,
    href: String,
    id: String,
    name: String,
    #[serde(rename(deserialize = "type"))]
    artist_type: String,
    uri: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Album {
    album_type: String,
    artists: Vec<Artist>,
    available_markets: Vec<String>,
    external_urls: ExternalUrls,
    href: String,
    id: String,
    images: Vec<Image>,
    name: String,
    release_date: String,
    release_date_precision: String,
    total_tracks: i32,
    #[serde(rename(deserialize = "type"))]
    play_type: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Image {
    height: Option<i32>,
    url: String,
    width: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Item {
    album: Album,
    artists: Vec<Artist>,
    available_markets: Vec<String>,
    disc_number: i32,
    duration_ms: i32,
    explicit: bool,
    external_ids: ExternalIds,
    external_urls: ExternalUrls,
    href: String,
    id: String,
    is_local: bool,
    name: String,
    popularity: i32,
    preview_url: Option<String>,
    track_number: i32,
    #[serde(rename(deserialize = "type"))]
    item_type: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct ExternalIds {
    isrc: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Context {
    external_urls: ExternalUrls,
    href: String,
    #[serde(rename(deserialize = "type"))]
    context_type: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PlaybackState {
    device: Device,
    shuffle_state: bool,
    repeat_state: String,
    timestamp: i64,
    context: Context,
    progress_ms: i32,
    item: Option<Item>,
}

impl PlaybackState {
    pub fn get_progress(&self) -> i32 {
        return self.progress_ms;
    }

    pub fn progress_as_string(&self) -> String {
        // precision of minutes and seconds is 2
        let progress = self.get_progress();
        let seconds = progress / 1000;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        return format!("{:0>2}:{:0>2}", minutes, seconds);
    }

    pub fn is_diff(&self, other: &PlaybackState) -> bool {
        return self.item.as_ref().unwrap().id != other.item.as_ref().unwrap().id;
    }

    fn get_currently_playing(&self) -> (String, String) {
        let item = self.item.as_ref().unwrap();
        return (item.name.clone(), item.artists[0].name.clone());
    }
}

impl Display for PlaybackState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Currently playing: {} by {}",
            self.item.as_ref().unwrap().name,
            self.item.as_ref().unwrap().artists[0].name
        )
    }
}
