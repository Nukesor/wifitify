use std::io::Cursor;

use bytes::Buf;

use crate::payload::extractors::supported_rates::supported_rates;
use crate::payload::FromBytes;

#[derive(Clone, Debug)]
pub struct AssociationResponse {
    pub cap_info: u16,
    pub status_code: u16,
    pub association_id: u16,
    pub supported_rates: Vec<f32>,
}

impl FromBytes for AssociationResponse {
    fn from_bytes(input: &[u8]) -> AssociationResponse {
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let status_code = cursor.get_u16_le();
        let association_id = cursor.get_u16_le();

        AssociationResponse {
            cap_info,
            status_code,
            association_id,
            supported_rates: supported_rates(cursor.bytes()),
        }
    }
}
