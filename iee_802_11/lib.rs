use anyhow::Result;

pub mod body;
pub mod header;

use crate::body::info::*;
use crate::header::*;

/// This represents a full IEE 800.11 frame.
/// It's devided into the Header,
pub struct Frame {
    pub header: Header,
    pub body: BodyInformation,
    //frame_check_sequence: [u8; 4],
}

impl Frame {
    pub fn from_bytes(input: &[u8]) -> Result<Frame> {
        let (header, body_bytes) = Header::from_bytes(input)?;

        let body = if !body_bytes.is_empty() {
            Frame::parse_body(&header.frame_control, &body_bytes)
        } else {
            BodyInformation::Empty
        };

        Ok(Frame { header, body })
    }

    fn parse_body(frame_control: &FrameControl, input: &[u8]) -> BodyInformation {
        // For now, only management Frames are handled
        if !matches!(frame_control.frame_type, FrameType::Management) {
            return BodyInformation::UnHandled(true);
        }

        // Check which kind of frame sub-type we got
        match frame_control.frame_subtype {
            FrameSubType::Beacon => BodyInformation::Beacon(Beacon::from_bytes(input)),
            FrameSubType::ProbeReq => {
                BodyInformation::ProbeRequest(ProbeRequest::from_bytes(input))
            }
            FrameSubType::ProbeResp => {
                BodyInformation::ProbeResponse(ProbeResponse::from_bytes(input))
            }
            FrameSubType::AssoReq => {
                BodyInformation::AssociationRequest(AssociationRequest::from_bytes(input))
            }
            FrameSubType::AssoResp => {
                BodyInformation::AssociationResponse(AssociationResponse::from_bytes(input))
            }
            _ => BodyInformation::UnHandled(true),
        }
    }
}
