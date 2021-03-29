use anyhow::Result;

pub mod frame_control;
pub mod frame_types;
pub mod payload;

use crate::frame_control::FrameControl;
use crate::frame_types::*;
use crate::payload::variants::*;
use crate::payload::*;

/// This represents a full IEE 800.11 frame.
/// It's devided into the Header,
pub struct Frame {
    pub control: FrameControl,
    pub payload: Payload,
    //frame_check_sequence: [u8; 4],
}

impl Frame {
    pub fn from_bytes(input: &[u8]) -> Result<Frame> {
        let frame_control = FrameControl::from_bytes(&input[0..2])?;
        let payload = Frame::parse_payload(&frame_control, &input[2..]);

        Ok(Frame {
            control: frame_control,
            payload,
        })
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
