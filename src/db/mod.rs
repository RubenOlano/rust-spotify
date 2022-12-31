use serde::Serialize;
use uuid::Uuid;

pub mod config;
pub mod songs;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Songs {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub youtube_id: String,
}
