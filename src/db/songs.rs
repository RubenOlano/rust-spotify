use std::sync::Arc;

use color_eyre::eyre::Result;
use spotify_music_vid::Song;
use sqlx::PgPool;
use tracing::instrument;

use super::Songs;

pub struct SongRepository {
    pool: Arc<PgPool>,
}

impl SongRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn create(&self, song: Song, song_id: &String) -> Result<()> {
        sqlx::query_as!(
            Songs,
            r#"
            insert into songs (title, ARTIST, YOUTUBE_ID)
            values ($1, $2, $3)
            returning *
            "#,
            song.name,
            song.artist,
            song_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(())
    }
    #[instrument(skip(self))]
    pub async fn get(&self, song: &Song) -> Option<String> {
        let song = sqlx::query_as::<_, Songs>(
            r#"
            SELECT * FROM songs
            WHERE title = $1 AND artist = $2
            "#,
        )
        .bind(song.name.to_string())
        .bind(song.artist.to_string())
        .fetch_one(&*self.pool)
        .await
        .ok()?;

        Some(song.youtube_id)
    }
}
