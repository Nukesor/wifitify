use anyhow::Result;
use sqlx::FromRow;

use std::collections::{HashMap, HashSet};

use crate::db::DbPool;

#[derive(FromRow)]
pub struct DeviceStation {
    pub device: i32,
    pub station: i32,
}

impl DeviceStation {
    pub async fn persist(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            "
INSERT INTO devices_stations
(device, station)
VALUES ($1, $2)
",
            self.device,
            self.station,
        )
        .fetch_one(pool)
        .await?;

        Ok(())
    }

    pub async fn get_station_device_map(pool: &DbPool) -> Result<HashMap<i32, HashSet<i32>>> {
        let rows = sqlx::query_as!(DeviceStation, "SELECT * FROM devices_stations")
            .fetch_all(pool)
            .await?;

        let mut map = HashMap::new();
        for row in rows {
            let set = map.entry(row.station).or_insert(HashSet::new());
            set.insert(row.device);
        }

        Ok(map)
    }
}
