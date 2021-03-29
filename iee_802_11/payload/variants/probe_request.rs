use std::io::Cursor;

use bytes::Buf;

use crate::payload::data::ssid::SSID;
use crate::payload::extractors::supported_rates::supported_rates;

#[derive(Clone, Debug)]
pub struct ProbeRequest {
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
}

impl ProbeRequest {
    pub fn from_bytes(input: &[u8]) -> ProbeRequest {
        let mut cursor = Cursor::new(input);

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        ProbeRequest {
            ssid,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
