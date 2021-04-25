use anyhow::Result;
use sqlx::FromRow;

use std::collections::HashMap;

use crate::db::types::MacAddress;
use crate::db::DbPool;

#[derive(FromRow)]
pub struct Station {
    pub id: i32,
    pub mac_address: MacAddress,
    pub ssid: Option<String>,
    pub nickname: Option<String>,
    pub description: Option<String>,
    pub channel: i32,
    pub watch: bool,
}

impl Station {
    pub async fn persist(&self, pool: &DbPool) -> Result<i32> {
        let record = sqlx::query!(
            "
INSERT INTO stations
(mac_address, ssid, nickname, description, channel)
VALUES ($1, $2, $3, $4, $5)
RETURNING id
",
            self.mac_address.to_string(),
            self.ssid.clone(),
            self.nickname.clone(),
            self.description.clone(),
            self.channel,
        )
        .fetch_one(pool)
        .await?;

        Ok(record.id)
    }

    pub async fn known_stations(pool: &DbPool) -> Result<HashMap<String, Station>> {
        let stations: Vec<Station> = sqlx::query_as!(
            Station,
            r#"
SELECT
    id,
    mac_address as "mac_address: MacAddress",
    ssid,
    nickname,
    description,
    channel,
    watch
FROM stations
"#
        )
        .fetch_all(pool)
        .await?;

        let mut station_map = HashMap::new();
        for station in stations {
            station_map.insert(station.mac_address.to_string(), station);
        }

        Ok(station_map)
    }
}
