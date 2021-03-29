use std::io::Cursor;

use bytes::Buf;

use crate::payload::data::ssid::SSID;
use crate::payload::extractors::supported_rates::supported_rates;

#[derive(Clone, Debug)]
pub struct AssociationRequest {
    pub cap_info: u16,
    pub interval: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
}

impl AssociationRequest {
    pub fn from_bytes(input: &[u8]) -> AssociationRequest {
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let interval = cursor.get_u16_le();
        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        AssociationRequest {
            cap_info,
            interval,
            ssid,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
