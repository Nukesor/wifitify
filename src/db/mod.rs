use anyhow::Result;
use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub mod models;
pub mod queries;
pub mod types;

pub type DbPool = Pool<Postgres>;

/// Create the connection pool for our application.
/// This pool will be passed around and used for every query in our applicaton.
pub async fn init_pool() -> Result<DbPool> {
    let pool_size = 5;

    info!("Spawn database pool with {} slots", pool_size);
    Ok(PgPoolOptions::new()
        .max_connections(pool_size)
        .connect("postgres://nuke@localhost/wifitify")
        .await?)
}
