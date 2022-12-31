use color_eyre::eyre::{Error, Result};
use futures_util::{stream::SplitSink, SinkExt};
use rspotify::{
    model::{AdditionalType, CurrentlyPlayingContext, Market, PlayableItem},
    prelude::OAuthClient,
    AuthCodeSpotify,
};
use spotify_music_vid::Song;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use warp::ws::{Message, WebSocket};

use crate::youtube_client::YoutubeClient;

type Writer = SplitSink<WebSocket, Message>;
#[derive(Debug)]
pub struct SpotifyClient {
    pub spotify: AuthCodeSpotify,
    yt_client: YoutubeClient,
    prev_state: Option<CurrentlyPlayingContext>,
    writer: Writer,
}

impl SpotifyClient {
    /// Creates a new [`SpotifyClient`].
    /// This function will also load the environment variables
    /// `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` are required
    pub fn new(auth: AuthCodeSpotify, writer: Writer) -> Result<Self> {
        info!("Creating new SpotifyClient and loading environment variables");
        let yt_client = YoutubeClient::new()?;

        Ok(Self {
            spotify: auth,
            yt_client,
            prev_state: None,
            writer,
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
        match res {
            Some(context) => Ok(context),
            None => {
                warn!("No response found");
                Err(Error::msg("No context"))
            }
        }
    }

    /// Returns the start polling of this [`SpotifyClient`].
    /// This function will check if the state has changed every 250 milliseconds.
    /// If the state has changed, it will send the video url to the client.
    pub async fn start_polling(&mut self) -> Result<()> {
        info!("Starting polling");
        while let Ok(state) = self.get_state_loop().await {
            if self.check_state_change(&state) {
                info!("State changed, opening video");
                self.handle_state_change(state).await?;
            }

            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    async fn handle_state_change(&mut self, state: CurrentlyPlayingContext) -> Result<()> {
        let song = Song::from_context(state)?;
        loop {
            let vid = self.yt_client.get_song_vid(&song).await;
            if let Ok(vid) = vid {
                self.send_video(Ok(vid)).await?;
                return Ok(());
            }
            self.send_video(vid).await?;
        }
    }

    async fn send_video(&mut self, vid: Result<String, Error>) -> Result<(), Error> {
        Ok(match vid {
            Ok(vid) => {
                self.writer.send(Message::text(vid)).await?;
                return Ok(());
            }
            Err(e) => {
                error!("Failed to get video: {e}, retrying in 2 seconds");
                sleep(Duration::from_secs(2)).await;
            }
        })
    }

    fn check_state_change(&mut self, state: &CurrentlyPlayingContext) -> bool {
        if self.prev_state.is_none() {
            self.update_state(state);
            return true;
        }

        let prev = match self.prev_state.as_ref() {
            Some(prev) => prev,
            None => {
                warn!("Previous state was None");
                return false;
            }
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

        if prev_track.name != curr_track.name {
            self.update_state(state);
            true
        } else {
            false
        }
    }

    fn update_state(&mut self, state: &CurrentlyPlayingContext) {
        self.prev_state = Some(state.clone());
    }
}
