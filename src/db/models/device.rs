use anyhow::Result;
use sqlx::FromRow;

use std::collections::HashMap;

use crate::db::types::MacAddress;
use crate::db::Connection;

#[derive(FromRow)]
pub struct Device {
    pub id: i32,
    pub mac_address: MacAddress,
    pub nickname: Option<String>,
    pub description: Option<String>,
    pub watch: bool,
}

impl Device {
    pub async fn get_by_mac<T: ToString>(
        connection: &mut Connection,
        mac_address: &T,
    ) -> Result<Option<Device>> {
        let device = sqlx::query_as!(
            Device,
            r#"
SELECT
    id,
    mac_address as "mac_address: MacAddress",
    nickname,
    description,
    watch
FROM devices
WHERE mac_address = $1
"#,
            mac_address.to_string()
        )
        .fetch_optional(&mut **connection)
        .await?;

        Ok(device)
    }

    pub async fn persist(&mut self, connection: &mut Connection) -> Result<i32> {
        let record = sqlx::query!(
            "
INSERT INTO devices
(mac_address, nickname, description)
VALUES ($1, $2, $3)
RETURNING id
",
            self.mac_address.to_string(),
            self.nickname.clone(),
            self.description.clone(),
        )
        .fetch_one(&mut **connection)
        .await?;

        self.id = record.id;
        Ok(record.id)
    }

    pub async fn known_devices(connection: &mut Connection) -> Result<HashMap<String, Device>> {
        let devices: Vec<Device> = sqlx::query_as!(
            Device,
            r#"
SELECT
    id,
    mac_address as "mac_address: MacAddress",
    nickname,
    description,
    watch
FROM devices"#
        )
        .fetch_all(&mut **connection)
        .await?;

        let mut device_map = HashMap::new();
        for device in devices {
            device_map.insert(device.mac_address.to_string(), device);
        }

        Ok(device_map)
    }
}
