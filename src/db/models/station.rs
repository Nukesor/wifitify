use anyhow::Result;
use libwifi::frame::components::MacAddress;
use sqlx::FromRow;

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
    pub async fn persist(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            "
INSERT INTO stations
(mac_address, ssid, nickname, description)
VALUES ($1, $2, $3, $4)",
            self.mac_address.to_string(),
            self.ssid.clone().unwrap_or("".to_string()),
            self.nickname.clone().unwrap_or("".to_string()),
            self.description.clone().unwrap_or("".to_string()),
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
