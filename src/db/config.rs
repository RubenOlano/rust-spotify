use std::time::Duration;

use color_eyre::eyre::Result;
use dotenv::dotenv;
use eyre::Context;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, instrument};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: i64,
    pub database_url: String,
    pub secret_key: String,
}
impl Config {
    #[instrument]
    pub fn from_env() -> Result<Self> {
        // dotenv().ok();

        info!("Loading config from environment variables");

        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
    #[instrument]
    pub async fn create_db_pool(&self) -> Result<PgPool> {
        info!("Creating database pool");
        PgPoolOptions::new()
            .idle_timeout(Duration::from_secs(30))
            .connect(&self.database_url)
            .await
            .context("Creating database pool")
    }
}
