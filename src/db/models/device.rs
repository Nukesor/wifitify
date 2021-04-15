use libwifi::components::MacAddress;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Device {
    pub mac_address: MacAddress,
    pub nickname: String,
    pub description: String,
    pub station_mac_address: MacAddress,
}
