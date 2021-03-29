/// Contains structs representing recurring sets structured data.
/// For instance, MAC-Addresses, default headers, etc.
pub mod data;
/// This contains helper functions that are used to interpret and extract information from a byte
/// array. These should only be used internally.
mod extractors;
/// Contains struct representations for all frame types/subtypes.
pub mod variants;

use variants::*;

use crate::frame_control::FrameControl;
use crate::frame_types::*;

#[derive(Clone, Debug)]
/// This represents all currently supported payloads for various frame types/subtypes.
/// Each variant is represented by its own struct, which can be found in the [variants] module.
pub enum Payload {
    Beacon(Beacon),
    ProbeRequest(ProbeRequest),
    ProbeResponse(ProbeResponse),
    AssociationRequest(AssociationRequest),
    AssociationResponse(AssociationResponse),
    UnHandled(bool),
    Empty,
}

impl Payload {
    pub fn parse(frame_control: &FrameControl, input: &[u8]) -> Payload {
        // For now, only management Frames are handled
        if !matches!(frame_control.frame_type, FrameType::Management) {
            return Payload::UnHandled(true);
        }

        // Check which kind of frame sub-type we got
        match frame_control.frame_subtype {
            FrameSubType::Beacon => Payload::Beacon(Beacon::parse(frame_control, input)),
            FrameSubType::ProbeReq => {
                Payload::ProbeRequest(ProbeRequest::parse(frame_control, input))
            }
            FrameSubType::ProbeResp => {
                Payload::ProbeResponse(ProbeResponse::parse(frame_control, input))
            }
            FrameSubType::AssoReq => {
                Payload::AssociationRequest(AssociationRequest::parse(frame_control, input))
            }
            FrameSubType::AssoResp => {
                Payload::AssociationResponse(AssociationResponse::parse(frame_control, input))
            }
            _ => Payload::UnHandled(true),
        }
    }
}
