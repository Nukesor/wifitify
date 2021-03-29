/// This contains helper functions that are used to interpret and extract information from a byte
/// array. These should only be used internally.
mod extractors;
/// The re
pub mod variants;

pub mod ssid;

use variants::*;

pub trait FromBytes {
    fn from_bytes(input: &[u8]) -> Self
    where
        Self: Sized;
}

#[derive(Clone, Debug)]
/// This frame represents all currently supported frame sub/types.
/// Each type is represented by its own struct, which can be found in the [variants] module.
pub enum Frame {
    Beacon(Beacon),
    ProbeRequest(ProbeRequest),
    ProbeResponse(ProbeResponse),
    AssociationRequest(AssociationRequest),
    AssociationResponse(AssociationResponse),
    UnHandled(bool),
    Empty,
}
