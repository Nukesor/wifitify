use anyhow::Result;
use chrono::Utc;
use sqlx::types::chrono::DateTime;
use sqlx::FromRow;

use crate::db::Connection;

#[derive(FromRow)]
pub struct Data {
    pub time: DateTime<Utc>,
    pub device: i32,
    pub station: i32,
    pub bytes_per_minute: i32,
}

impl Data {
    pub async fn persist(&self, connection: &mut Connection) -> Result<()> {
        sqlx::query!(
            "
INSERT INTO data (time, device, station, bytes_per_minute)
VALUES ($1, $2, $3, $4)
ON CONFLICT (time, device, station) DO
UPDATE SET bytes_per_minute = data.bytes_per_minute + $5",
            self.time,
            self.device,
            self.station,
            self.bytes_per_minute,
            self.bytes_per_minute,
        )
        .execute(&mut **connection)
        .await?;

        Ok(())
    }
}
