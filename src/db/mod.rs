use anyhow::Result;
use log::info;
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub mod models;
pub mod queries;
pub mod types;

pub type DbPool = Pool<Postgres>;
pub type Connection = PoolConnection<Postgres>;

/// Create the connection pool for our application.
/// This pool will be passed around and used for every query in our applicaton.
pub async fn init_pool(database_url: &str) -> Result<DbPool> {
    let pool_size = 80;

    info!("Spawn database pool with {} slots", pool_size);
    Ok(PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(database_url)
        .await?)
}
