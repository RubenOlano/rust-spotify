use serde::{Deserialize, Serialize};

pub mod config;
pub mod songs;

#[derive(Debug, Clone, sqlx::FromRow, sqlx::Decode, sqlx::Encode, Serialize, Deserialize)]
pub struct Songs {
    title: String,
    artist: String,
    youtube_id: String,
}
