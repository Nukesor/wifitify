use std::io::Cursor;

use bytes::Buf;

use crate::frame_control::FrameControl;
use crate::payload::data::Header;
use crate::payload::extractors::supported_rates;

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
