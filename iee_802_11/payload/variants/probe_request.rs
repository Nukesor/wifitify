use std::io::Cursor;

use bytes::Buf;

use crate::frame_control::FrameControl;
use crate::payload::data::{Header, SSID};
use crate::payload::extractors::supported_rates;

#[derive(Clone, Debug)]
pub struct ProbeRequest {
    pub header: Header,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
}

impl ProbeRequest {
    pub fn parse(frame_control: &FrameControl, input: &[u8]) -> ProbeRequest {
        let (header, input) = Header::parse(frame_control, input);
        let mut cursor = Cursor::new(input);

        let ssid = SSID::parse(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        ProbeRequest {
            header,
            ssid,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
