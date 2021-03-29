use anyhow::Result;

pub mod frame;
pub mod header;

use crate::frame::variants::*;
use crate::frame::*;
use crate::header::*;

/// This represents a full IEE 800.11 frame.
/// It's devided into the Header,
pub struct Message {
    pub header: Header,
    pub frame: Frame,
    //frame_check_sequence: [u8; 4],
}

impl Message {
    pub fn from_bytes(input: &[u8]) -> Result<Message> {
        let (header, frame_bytes) = Header::from_bytes(input)?;

        let frame = if !frame_bytes.is_empty() {
            Message::parse_frame(&header.frame_control, &frame_bytes)
        } else {
            Frame::Empty
        };

        Ok(Message { header, frame })
    }

    fn parse_frame(frame_control: &FrameControl, input: &[u8]) -> Frame {
        // For now, only management Frames are handled
        if !matches!(frame_control.frame_type, FrameType::Management) {
            return Frame::UnHandled(true);
        }

        // Check which kind of frame sub-type we got
        match frame_control.frame_subtype {
            FrameSubType::Beacon => Frame::Beacon(Beacon::from_bytes(input)),
            FrameSubType::ProbeReq => Frame::ProbeRequest(ProbeRequest::from_bytes(input)),
            FrameSubType::ProbeResp => Frame::ProbeResponse(ProbeResponse::from_bytes(input)),
            FrameSubType::AssoReq => {
                Frame::AssociationRequest(AssociationRequest::from_bytes(input))
            }
            FrameSubType::AssoResp => {
                Frame::AssociationResponse(AssociationResponse::from_bytes(input))
            }
            _ => Frame::UnHandled(true),
        }
    }
}
