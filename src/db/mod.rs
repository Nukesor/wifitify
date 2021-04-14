use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

type DbPool = Pool<Postgres>;

/// Create the connection pool for our application.
/// This pool will be passed around and used for every query in our applicaton.
pub async fn init_pool() -> Result<DbPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://localhost/sniffer")
        .await?)
}
