/// Contains structs representing recurring sets structured data.
/// For instance, MAC-Addresses, default headers, etc.
pub mod data;
/// This contains helper functions that are used to interpret and extract information from a byte
/// array. These should only be used internally.
mod extractors;
/// Contains struct representations for all frame types/subtypes.
pub mod variants;

use variants::*;

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
