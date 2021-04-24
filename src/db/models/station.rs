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
}

impl Station {
    pub async fn persist(&self, pool: &DbPool) -> Result<i32> {
        let record = sqlx::query!(
            "
INSERT INTO stations
(mac_address, ssid, nickname, description)
VALUES ($1, $2, $3, $4)
RETURNING id
",
            self.mac_address.to_string(),
            self.ssid.clone(),
            self.nickname.clone(),
            self.description.clone(),
        )
        .fetch_one(pool)
        .await?;

        Ok(record.id)
    }

    pub async fn known_macs(pool: &DbPool) -> Result<HashMap<String, i32>> {
        let rows = sqlx::query!("SELECT id, mac_address FROM stations")
            .fetch_all(pool)
            .await?;

        let mut macs = HashMap::new();
        for row in rows {
            println!("Row: {:?}", row);
            macs.insert(row.mac_address, row.id);
        }

        Ok(macs)
    }
}
