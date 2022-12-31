use std::sync::Arc;

use color_eyre::eyre::{Error, Result};
use futures_util::{stream::SplitSink, SinkExt};
use rspotify::{
    model::{AdditionalType, CurrentlyPlayingContext, Market, PlayableItem},
    prelude::OAuthClient,
    AuthCodeSpotify,
};
use spotify_music_vid::Song;
use sqlx::{Pool, Postgres};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use warp::ws::{Message, WebSocket};

use crate::{db::songs::SongRepository, youtube_client::YoutubeClient};

type Writer = SplitSink<WebSocket, Message>;
pub struct SpotifyClient {
    pub spotify: AuthCodeSpotify,
    yt_client: YoutubeClient,
    prev_state: Option<CurrentlyPlayingContext>,
    writer: Writer,
    db_pool: SongRepository,
}

impl SpotifyClient {
    /// Creates a new [`SpotifyClient`].
    /// This function will also load the environment variables
    /// `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` are required
    pub fn new(auth: AuthCodeSpotify, writer: Writer, pool: Pool<Postgres>) -> Result<Self> {
        info!("Creating new SpotifyClient and loading environment variables");
        let yt_client = YoutubeClient::new()?;
        let pool = SongRepository::new(Arc::new(pool));

        Ok(Self {
            spotify: auth,
            yt_client,
            prev_state: None,
            writer,
            db_pool: pool,
        })
    }

    async fn get_state_loop(&mut self) -> Result<CurrentlyPlayingContext> {
        let mut state = self.get_state().await;
        while let Err(ref e) = state {
            println!("Something went wrong, retrying in 5 seconds");
            error!("Failed to get state: {e}");
            sleep(Duration::from_secs(5)).await;
            state = self.get_state().await;
        }
        state
    }

    async fn get_state(&self) -> Result<CurrentlyPlayingContext> {
        let market = Market::Country(rspotify::model::Country::UnitedStates);

        let add = AdditionalType::Track;

        let res = self
            .spotify
            .current_playing(Some(market), Some(vec![&add]))
            .await?;
        res.map_or_else(
            || {
                warn!("No song is currently playing");
                Err(color_eyre::eyre::eyre!("No song is currently playing"))
            },
            Ok,
        )
    }

    /// Returns the start polling of this [`SpotifyClient`].
    /// This function will check if the state has changed every 250 milliseconds.
    /// If the state has changed, it will send the video url to the client.
    pub async fn start_polling(&mut self) -> Result<()> {
        info!("Starting polling");
        while let Ok(state) = self.get_state_loop().await {
            if self.check_state_change(&state) {
                info!("State changed, sending video");
                self.handle_state_change(state).await?;
            }

            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    async fn handle_state_change(&mut self, state: CurrentlyPlayingContext) -> Result<()> {
        let song = Song::from_context(state)?;
        info!("Checking if song is in database");
        if let Some(song_id) = self.db_pool.get(&song).await {
            info!("Song is in database, sending video");
            let url = Song::get_url_with_duration(&song_id, &song.progress.to_string());
            self.send_video(Ok(url)).await?;
            return Ok(());
        }

        let vid = self.yt_client.get_song_vid(&song).await;
        if let Ok((url, id)) = vid {
            info!("Song is not in database, adding to database");
            self.db_pool.create(song, &id).await?;
            self.send_video(Ok(url)).await?;
            return Ok(());
        }

        Ok(())
    }

    async fn send_video(&mut self, vid: Result<String, Error>) -> Result<(), Error> {
        let msg = match vid {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to get video: {e}");
                "Failed to get video".to_string()
            }
        };

        self.writer.send(Message::text(msg)).await?;
        Ok(())
    }

    fn check_state_change(&mut self, state: &CurrentlyPlayingContext) -> bool {
        let prev = if let Some(prev) = self.prev_state.as_ref() {
            prev
        } else {
            warn!("Previous state was None");
            return false;
        };

        let prev_item = match &prev.item {
            Some(item) => item,
            None => {
                warn!("Previous item was None");
                return false;
            }
        };

        let curr_item = match &state.item {
            Some(item) => item,
            None => {
                warn!("Current item was None");
                return false;
            }
        };

        let prev_track = match prev_item {
            PlayableItem::Track(track) => track,
            PlayableItem::Episode(_) => {
                warn!("Previous item was an episode");
                return false;
            }
        };

        let curr_track = match curr_item {
            PlayableItem::Track(track) => track,
            PlayableItem::Episode(_) => {
                warn!("Current item was an episode");
                return false;
            }
        };

        if prev_track.name == curr_track.name {
            false
        } else {
            self.update_state(state);
            true
        }
    }

    fn update_state(&mut self, state: &CurrentlyPlayingContext) {
        self.prev_state = Some(state.clone());
    }
}
