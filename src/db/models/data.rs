use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(FromRow)]
pub struct Data {
    pub time: NaiveDateTime,
    pub device: i32,
    pub station: i32,
    pub frame_type: String,
    pub amount_per_minute: i32,
}

impl Data {
    pub async fn persist(&self, pool: DbPool) -> Result<()> {
        sqlx::query(
            "
INSERT INTO data (device, station, time, frame_type, amount_per_minute)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT DO
UPDATE SET amount_per_minute = amount_per_minute + $5",
        )
        .bind(self.device)
        .bind(self.station)
        .bind(self.time)
        .bind(self.frame_type.to_string())
        .bind(self.amount_per_minute)
        .execute(&pool)
        .await?;

        Ok(())
    }
}
