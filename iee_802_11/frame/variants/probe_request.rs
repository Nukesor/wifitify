use std::io::Cursor;

use bytes::Buf;

use crate::frame::extractors::supported_rates::supported_rates;
use crate::frame::ssid::SSID;
use crate::frame::FromBytes;

#[derive(Clone, Debug)]
pub struct ProbeRequest {
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
}

impl FromBytes for ProbeRequest {
    fn from_bytes(input: &[u8]) -> ProbeRequest {
        let mut cursor = Cursor::new(input);

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        ProbeRequest {
            ssid,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
