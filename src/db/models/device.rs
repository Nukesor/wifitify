use sqlx::FromRow;

use crate::db::types::MacAddress;

#[derive(FromRow)]
pub struct Device {
    pub id: i32,
    pub mac_address: MacAddress,
    pub nickname: String,
    pub description: String,
    pub station_mac_address: MacAddress,
}
