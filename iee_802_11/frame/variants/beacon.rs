use std::io::Cursor;

use bytes::Buf;

use crate::frame::extractors::country::*;
use crate::frame::extractors::supported_rates::supported_rates;
use crate::frame::ssid::SSID;
use crate::frame::FromBytes;

#[derive(Clone, Debug)]
pub struct Beacon {
    pub timestamp: u64,
    pub interval: u16,
    pub cap_info: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
    pub current_channel: u8,
    pub country: Country,
}

impl FromBytes for Beacon {
    fn from_bytes(input: &[u8]) -> Beacon {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2); // 2 accounts for Id + Len

        let supported_rates = supported_rates(cursor.bytes());
        cursor.advance(supported_rates.len() + 2); // 2 accounts for Id + Len
        let info = get_info(cursor.bytes());

        Beacon {
            timestamp,
            interval,
            cap_info,
            ssid,
            supported_rates,
            current_channel: info.current_channel,
            country: info.country,
        }
    }
}
