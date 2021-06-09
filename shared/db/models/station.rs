use anyhow::Result;
use sqlx::FromRow;

use std::collections::HashMap;

use crate::db::types::MacAddress;
use crate::db::Connection;

#[derive(FromRow)]
pub struct Station {
    pub id: i32,
    pub mac_address: MacAddress,
    pub ssid: Option<String>,
    pub channel: i32,
    pub power_level: Option<i32>,
    pub nickname: Option<String>,
    pub description: Option<String>,
    pub watch: bool,
}

impl Station {
    pub async fn get_by_mac<T: ToString>(
        connection: &mut Connection,
        mac_address: &T,
    ) -> Result<Option<Self>> {
        let record = sqlx::query_as!(
            Station,
            r#"
SELECT
    id,
    mac_address as "mac_address: MacAddress",
    ssid,
    channel,
    power_level,
    watch,
    nickname,
    description
FROM stations
WHERE mac_address = $1
"#,
            mac_address.to_string(),
        )
        .fetch_optional(connection)
        .await?;

        Ok(record)
    }

    pub async fn persist(&mut self, connection: &mut Connection) -> Result<i32> {
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
        .fetch_one(connection)
        .await?;

        self.id = record.id;
        Ok(record.id)
    }

    pub async fn update_metadata(&self, connection: &mut Connection) -> Result<()> {
        sqlx::query!(
            "
UPDATE stations
    SET ssid = $2,
    channel = $3,
    power_level = $4
WHERE id = $1
",
            self.id,
            self.ssid.clone(),
            self.channel,
            self.power_level,
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    pub async fn known_stations(connection: &mut Connection) -> Result<HashMap<String, Station>> {
        let stations: Vec<Station> = sqlx::query_as!(
            Station,
            r#"
SELECT
    id,
    mac_address as "mac_address: MacAddress",
    ssid,
    channel,
    power_level,
    watch,
    nickname,
    description
FROM stations
"#
        )
        .fetch_all(connection)
        .await?;

        let mut station_map = HashMap::new();
        for station in stations {
            station_map.insert(station.mac_address.to_string(), station);
        }

        Ok(station_map)
    }
}
