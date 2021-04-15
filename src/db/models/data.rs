use libwifi::components::MacAddress;
use sqlx::FromRow;
use Chrono::DateTime;

#[derive(FromRow)]
pub struct Data {
    pub device: MacAddress,
    pub station: MacAddress,
    pub time: DateTime,
    pub frame_type: String,
    pub station: MacAddress,
}
