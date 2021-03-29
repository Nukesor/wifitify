use std::io::Cursor;

use bytes::Buf;

use crate::frame_control::FrameControl;
use crate::payload::data::{Header, SSID};
use crate::payload::extractors::supported_rates::supported_rates;

#[derive(Clone, Debug)]
pub struct AssociationRequest {
    pub header: Header,
    pub cap_info: u16,
    pub interval: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>,
}

impl AssociationRequest {
    pub fn parse(frame_control: &FrameControl, input: &[u8]) -> AssociationRequest {
        let (header, input) = Header::parse(frame_control, input);
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let interval = cursor.get_u16_le();
        let ssid = SSID::parse(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        AssociationRequest {
            header,
            cap_info,
            interval,
            ssid,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
