use anyhow::Result;

pub mod header;
pub mod payload;

use crate::header::*;
use crate::payload::variants::*;
use crate::payload::*;

/// This represents a full IEE 800.11 frame.
/// It's devided into the Header,
pub struct Frame {
    pub header: Header,
    pub payload: Payload,
    //frame_check_sequence: [u8; 4],
}

impl Frame {
    pub fn from_bytes(input: &[u8]) -> Result<Frame> {
        let (header, payload_bytes) = Header::from_bytes(input)?;

        let payload = if !payload_bytes.is_empty() {
            Frame::parse_payload(&header.frame_control, &payload_bytes)
        } else {
            Payload::Empty
        };

        Ok(Frame { header, payload })
    }

    fn parse_payload(frame_control: &FrameControl, input: &[u8]) -> Payload {
        // For now, only management Frames are handled
        if !matches!(frame_control.frame_type, FrameType::Management) {
            return Payload::UnHandled(true);
        }

        // Check which kind of frame sub-type we got
        match frame_control.frame_subtype {
            FrameSubType::Beacon => Payload::Beacon(Beacon::from_bytes(input)),
            FrameSubType::ProbeReq => Payload::ProbeRequest(ProbeRequest::from_bytes(input)),
            FrameSubType::ProbeResp => Payload::ProbeResponse(ProbeResponse::from_bytes(input)),
            FrameSubType::AssoReq => {
                Payload::AssociationRequest(AssociationRequest::from_bytes(input))
            }
            FrameSubType::AssoResp => {
                Payload::AssociationResponse(AssociationResponse::from_bytes(input))
            }
            _ => Payload::UnHandled(true),
        }
    }
}
