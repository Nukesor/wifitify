use std::io::Cursor;

use bytes::Buf;

use crate::frame_control::FrameControl;
use crate::payload::data::*;
use crate::payload::extractors::supported_rates;
use crate::payload::Addresses;

#[derive(Clone, Debug)]
pub struct AssociationResponse {
    pub header: Header,
    pub cap_info: u16,
    pub status_code: u16,
    pub association_id: u16,
    pub supported_rates: Vec<f32>,
}

impl AssociationResponse {
    pub fn parse(frame_control: &FrameControl, input: &[u8]) -> AssociationResponse {
        let (header, input) = Header::parse(frame_control, input);
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let status_code = cursor.get_u16_le();
        let association_id = cursor.get_u16_le();

        AssociationResponse {
            header,
            cap_info,
            status_code,
            association_id,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}

impl Addresses for AssociationResponse {
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
