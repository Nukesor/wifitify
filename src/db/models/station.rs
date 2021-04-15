use libwifi::components::MacAddress;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Station {
    pub mac_address: MacAddress,
    pub ssid: String,
    pub nickname: String,
    pub description: String,
}
