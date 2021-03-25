use std::io::Cursor;

use anyhow::{bail, Result};
use bytes::Buf;

use super::types::*;

#[inline]
/// Mini helper to check, whether a bit is set or not.
fn flag_is_set(data: u8, bit: u8) -> bool {
    if bit == 0 {
        let mask = 1;
        (data & mask) > 0
    } else {
        let mask = 1 << bit;
        (data & mask) > 0
    }
}

/// The very first two bytes of every frame contain the FrameControl header.
/// https://en.wikipedia.org/wiki/802.11_Frame_Types
///
/// The bytes are structured as follows.
/// The bytes will be interpreted from left to right.
///
/// First byte
/// bit 0-1: Version
/// bit 2-3: FrameType
/// bit 4-7: FrameSubType
///
/// 8 Flags (second byte)
/// bit_0 `to_ds`: Set if the frame is to be sent by the AP to the distribution system.
/// bit_1 `from_ds`: Set if the frame is from the distribution system.
/// bit_2 `more_frag`: Set if this frame is a fragment of a bigger frame and there are more fragments to follow.
/// bit_3 `retry`: Set if this frame is a retransmission, maybe through the loss of an ACK.
/// bit_4 `power_mgmt`: indicates what power mode ('save' or 'active') the station is to be in once the frame has been sent.
/// bit_5 `more_data`: set by the AP to indicate that more frames are destined to a particular station that may be in power save mode.
///                     These frames will be buffered at the AP ready for the station should it decide to become 'active'.
/// bit_6 `wep`: Set if WEP is being used to encrypt the body of the frame
/// bit_7 `order`: Set if the frame is being sent according to the 'Strictly Ordered Class'
#[derive(Copy, Clone, Debug)]
pub struct FrameControl {
    pub frame_type: FrameType,
    pub frame_subtype: FrameSubType,
    pub to_ds: bool,
    pub from_ds: bool,
    pub more_frag: bool,
    pub retry: bool,
    pub pwr_mgmt: bool,
    pub more_data: bool,
    pub wep: bool,
    pub order: bool,
}

impl FrameControl {
    /// This function parses a given two-byte frame-control header.
    pub fn from_bytes(input: &[u8]) -> Result<FrameControl> {
        let mut cursor = Cursor::new(input);

        // The first byte contains all protocol and FrameType information
        let version_type_subtype = cursor.get_u8();

        // The first two bits specify the protocol version
        // Until now, this has always been 0 and is expected to be 0
        if FrameControl::protocol_version(version_type_subtype) != 0 {
            bail!("Unknow protocol version");
        }

        // The next two bits determine what kind of frame we got
        let frame_type = FrameControl::frame_type(version_type_subtype);

        // The next 4 bits are then used to determine the frame sub-type.
        // The sub-type depends on the current FrameType
        let frame_subtype = match frame_type {
            FrameType::Management => FrameControl::management_frame_subtype(version_type_subtype),
            FrameType::Control => FrameControl::control_frame_subtype(version_type_subtype),
            FrameType::Data => FrameControl::data_frame_subtype(version_type_subtype),
            FrameType::Unknown => FrameSubType::UnHandled,
        };

        let flags = cursor.get_u8();
        let fc = FrameControl {
            frame_type,
            frame_subtype,
            to_ds: flag_is_set(flags, 0),
            from_ds: flag_is_set(flags, 1),
            more_frag: flag_is_set(flags, 2),
            retry: flag_is_set(flags, 3),
            pwr_mgmt: flag_is_set(flags, 4),
            more_data: flag_is_set(flags, 5),
            wep: flag_is_set(flags, 6),
            order: flag_is_set(flags, 7),
        };

        Ok(fc)
    }

    fn protocol_version(byte: u8) -> u8 {
        byte & 0b0000_0011
    }

    /// Get the FrameType from bit 3-4
    fn frame_type(byte: u8) -> FrameType {
        match (byte & 0b0000_1100) >> 2 {
            0 => FrameType::Management,
            1 => FrameType::Control,
            2 => FrameType::Data,
            _ => FrameType::Unknown,
        }
    }

    /// Get the FrameSubType from bit 4-7 under the assumption
    /// that this is a management frame.
    fn management_frame_subtype(byte: u8) -> FrameSubType {
        match byte >> 4 {
            0 => FrameSubType::AssoReq,
            1 => FrameSubType::AssoResp,
            2 => FrameSubType::ReassoReq,
            3 => FrameSubType::ReassoResp,
            4 => FrameSubType::ProbeReq,
            5 => FrameSubType::ProbeResp,
            8 => FrameSubType::Beacon,
            9 => FrameSubType::Atim,
            10 => FrameSubType::Disasso,
            11 => FrameSubType::Auth,
            12 => FrameSubType::Deauth,
            _ => FrameSubType::UnHandled,
        }
    }

    /// Get the FrameSubType from bit 4-7 under the assumption
    /// that this is a control frame.
    fn control_frame_subtype(byte: u8) -> FrameSubType {
        match byte >> 4 {
            0 => FrameSubType::Reserved,
            1 => FrameSubType::Reserved,
            2 => FrameSubType::Trigger,
            3 => FrameSubType::Tack,
            4 => FrameSubType::BeamformingReportPoll,
            5 => FrameSubType::NdpAnnouncement,
            6 => FrameSubType::ControlFrameExtension,
            7 => FrameSubType::ControlWrapper,
            8 => FrameSubType::BlockAckRequest,
            9 => FrameSubType::BlockAck,
            10 => FrameSubType::PsPoll,
            11 => FrameSubType::RTS,
            12 => FrameSubType::CTS,
            13 => FrameSubType::ACK,
            14 => FrameSubType::CfEnd,
            15 => FrameSubType::CfEndCfAck,
            _ => FrameSubType::UnHandled,
        }
    }

    /// Get the FrameSubType from bit 4-7 under the assumption
    /// that this is a data frame.
    fn data_frame_subtype(byte: u8) -> FrameSubType {
        match byte >> 4 {
            0 => FrameSubType::Data,
            1 => FrameSubType::DataCfAck,
            2 => FrameSubType::DataCfPull,
            3 => FrameSubType::DataCfAckCfPull,
            4 => FrameSubType::NullData,
            5 => FrameSubType::CfAck,
            6 => FrameSubType::CfPull,
            7 => FrameSubType::CfAckCfPull,
            8 => FrameSubType::QoS,
            10 => FrameSubType::QoSCfPull,
            11 => FrameSubType::QoSCfAckCfPull,
            12 => FrameSubType::QoSNullData,
            13 => FrameSubType::Reserved,
            _ => FrameSubType::UnHandled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag_for_bit(bit: u8, frame_control: &FrameControl) -> bool {
        match bit {
            0 => frame_control.to_ds,
            1 => frame_control.from_ds,
            2 => frame_control.more_frag,
            3 => frame_control.retry,
            4 => frame_control.pwr_mgmt,
            5 => frame_control.more_data,
            6 => frame_control.wep,
            7 => frame_control.order,
            _ => panic!("Unhandled bit {}", bit),
        }
    }

    #[test]
    /// Set each flag once and ensure that only that bit is set.
    /// For this, we shift a byte with value `1` up to seven times to the left.
    fn test_flags() {
        for bit in 0..7 {
            let second_byte = 0b0000_0001 << bit;
            let bytes = [0b0000_0000, second_byte];
            let frame_control = FrameControl::from_bytes(&bytes).unwrap();

            // All bits except the currently selected bit should be false.
            for check_bit in 0..7 {
                if bit == check_bit {
                    assert!(flag_for_bit(check_bit, &frame_control));
                } else {
                    assert!(!flag_for_bit(check_bit, &frame_control));
                }
            }
        }
    }

    /// Create a beacon frame control
    /// FrameType should be `00` and SubType `1000`
    /// Remember
    fn test_beacon() {
        let bytes = [0b1000_0000, 0b0000_0000];
        let frame_control = FrameControl::from_bytes(&bytes).unwrap();
    }
}
