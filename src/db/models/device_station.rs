use anyhow::Result;
use sqlx::FromRow;

use std::collections::{HashMap, HashSet};

use crate::db::Connection;

#[derive(FromRow)]
pub struct DeviceStation {
    pub device: i32,
    pub station: i32,
}

impl DeviceStation {
    pub async fn persist(&self, connection: &mut Connection) -> Result<()> {
        sqlx::query!(
            "
INSERT INTO devices_stations
(device, station)
VALUES ($1, $2)
ON CONFLICT DO NOTHING
",
            self.device,
            self.station,
        )
        .execute(&mut **connection)
        .await?;

        Ok(())
    }

    pub async fn get_by_station_device(
        connection: &mut Connection,
        station_id: i32,
        device_id: i32,
    ) -> Result<Option<Self>> {
        let record = sqlx::query_as!(
            DeviceStation,
            r#"
SELECT *
FROM devices_stations
WHERE station = $1 AND device = $2
"#,
            station_id,
            device_id,
        )
        .fetch_optional(&mut **connection)
        .await?;

        Ok(record)
    }

    pub async fn get_station_device_map(
        connection: &mut Connection,
    ) -> Result<HashMap<i32, HashSet<i32>>> {
        let rows = sqlx::query_as!(DeviceStation, "SELECT * FROM devices_stations")
            .fetch_all(&mut **connection)
            .await?;

        let mut map = HashMap::new();
        for row in rows {
            let set = map.entry(row.station).or_insert_with(HashSet::new);
            set.insert(row.device);
        }

        Ok(map)
    }
}
