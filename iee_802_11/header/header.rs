use anyhow::{bail, Result};

use super::frame_control::*;
use super::mac::*;

/// **Bytes 0-1**
///
/// These contain protocol meta information and flags.
/// Take a look at the [FrameControl] struct for more information.
///
/// **Bytes 2-3**
///
/// Those are the duration bytes.
/// They are quite specific and not explained here.
///
/// **Bytes 4-29**
///
/// These contain all important address information.
///
/// byte 4-9: Address 1
/// byte 10-15: Address 2
/// byte 16-21: Address 3
/// byte 22-23: Sequence Control
/// byte 24-29: Address 4
///
/// Which address is used in which way, depends on two flags in the FrameControl header.
///
/// Address 1:
/// The recipient station address on the BSS.
/// If `to_ds` is set, this is the AP address.
/// If `from_ds` is set then this is the station address
///
/// Address 2:
/// The transmitter station address on the BSS.
/// If `from_ds` is set, this is the AP address.
/// If `to_ds` is set then this is the station address.
///
/// Address 3:
/// If Address 1 contains the destination address then Address 3 will contain the source address.
/// Similarly, if Address 2 contains the source address then Address 3 will contain the destination address.
///
/// Sequence Control:
/// Contains the FragmentNumber and SequenceNumber that define the main frame and the number of fragments in the frame.
///
/// Address 4:
/// This is only set if a Wireless Distribution System (WDS) is being used (with AP to AP communication)
/// Address 1 contains the receiving AP address.
/// Address 2 contains the transmitting AP address.
/// Address 3 contains the destination station address.
/// Address 4 contains the source station address.
#[derive(Clone, Debug)]
pub struct Header {
    pub frame_control: FrameControl,
    pub duration: [u8; 2],
    pub address_1: MacAddress,
    pub address_2: MacAddress,
    pub address_3: MacAddress,
    pub address_4: Option<MacAddress>,
    pub seq_ctl: [u8; 2],
}

impl Header {
    pub fn from_bytes(input: &[u8]) -> Result<(Header, &[u8])> {
        // Parse the frame control header
        let frame_control = FrameControl::from_bytes(&input[0..2])?;

        // Read the duration. Bytes 2-3.
        // We don't do anything with this yet.
        let mut duration: [u8; 2] = [0; 2];
        duration.clone_from_slice(&input[2..4]);

        let address_1 = MacAddress::from_slice(&input[4..10]);
        let address_2 = MacAddress::from_slice(&input[10..16]);
        let address_3 = MacAddress::from_slice(&input[16..22]);

        let mut seq_ctl: [u8; 2] = [0; 2];
        seq_ctl.clone_from_slice(&input[22..24]);

        // The forth address is optional
        // Depending on whether it exists or not, the body begins at byte 24 or byte 30
        let mut address_4 = None;
        let mut data = &input[24..];
        if frame_control.to_ds && frame_control.from_ds {
            address_4 = Some(MacAddress::from_slice(&input[24..30]));
            data = &input[30..];
        }

        let header = Header {
            frame_control,
            duration,
            address_1,
            address_2,
            address_3,
            address_4,
            seq_ctl,
        };

        Ok((header, data))
    }

    /// Return the mac address of the sender
    pub fn src(&self) -> &MacAddress {
        if self.frame_control.to_ds && self.frame_control.from_ds {
            // This should be safe.
            // If both to_ds and from_ds are true, we always read the forth address.
            self.address_4.as_ref().unwrap()
        } else if self.frame_control.to_ds {
            &self.address_3
        } else if self.frame_control.from_ds {
            &self.address_1
        } else {
            &self.address_2
        }
    }

    /// Return the mac address of the receiver.
    /// A full `ff:ff:..` usually indicates a undirected broadcast.
    pub fn dest(&self) -> &MacAddress {
        if self.frame_control.to_ds && self.frame_control.from_ds {
            &self.address_3
        } else if self.frame_control.to_ds {
            &self.address_2
        } else if self.frame_control.from_ds {
            &self.address_3
        } else {
            &self.address_1
        }
    }

    /// The BSSID for this request.
    /// In most cases, this is expected to be present.
    /// The only time it's not, is in a wireless distributed system (WDS).
    pub fn bssid(&self) -> Option<&MacAddress> {
        if self.frame_control.to_ds && self.frame_control.from_ds {
            None
        } else if self.frame_control.to_ds {
            Some(&self.address_1)
        } else if self.frame_control.from_ds {
            Some(&self.address_2)
        } else {
            Some(&self.address_3)
        }
    }
}
