use libwifi::frame::components::MacAddress;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Station {
    pub id: i32,
    pub mac_address: MacAddress,
    pub ssid: String,
    pub nickname: String,
    pub description: String,
}
