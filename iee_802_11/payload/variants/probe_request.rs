use std::io::Cursor;

use bytes::Buf;

use crate::frame_control::FrameControl;
use crate::payload::data::*;
use crate::payload::extractors::supported_rates;
use crate::payload::Addresses;

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

impl Addresses for ProbeRequest {
    /// Returns the sender of the Frame.
    /// This isn't always send in every frame (e.g. CTS).
    fn src(&self) -> Option<&MacAddress> {
        Some(self.header.src())
    }

    fn dest(&self) -> &MacAddress {
        self.header.dest()
    }

    /// This isn't always send in every frame (e.g. RTS).
    fn bssid(&self) -> Option<&MacAddress> {
        self.header.bssid()
    }
}
