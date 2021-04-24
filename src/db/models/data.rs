use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(FromRow)]
pub struct Data {
    pub time: NaiveDateTime,
    pub device: i32,
    pub station: i32,
    pub bytes_per_minute: i32,
}

impl Data {
    pub async fn persist(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            "
INSERT INTO data (time, device, station, bytes_per_minute)
VALUES ($1, $2, $3, $4)
ON CONFLICT (time, device, station) DO
UPDATE SET bytes_per_minute = EXCLUDED.bytes_per_minute + $4",
            self.time,
            self.device,
            self.station,
            self.bytes_per_minute,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
