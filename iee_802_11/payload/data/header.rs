use anyhow::Result;

use super::mac::*;
use crate::frame_control::*;

/// This struct tries to represent a "normal" frame header to the best of it's abilities.
///
/// The IEE 802.11 frame specification is quite weird.
/// The formats vary wildly, the size can span from 10 bytes to 34 bytes and the way all of this is
/// interpreted depends on various flags, types.
///
/// This struct is designed to represent the full header!
/// The following is the aproximated format of a full header.
///
/// **Bytes 0-1** \
/// These contain protocol meta information and flags. These are always present!
/// Take a look at the [FrameControl] struct for more information.
///
/// **Bytes 2-3** \
/// Those are the duration bytes. These are always present!
/// They are quite specific and not explained here.
///
/// **Bytes 4-29** \
/// These contain all important address information.
///
/// byte 4-9: Address 1. Always present!
/// byte 10-15: Address 2. May be missing.
/// byte 16-21: Address 3. May be missing.
/// byte 22-23: Sequence Control. May be missing.
/// byte 24-29: Address 4. May be missing.
///
/// Which address is used in which way, depends on a combination of
/// - two flags in the FrameControl header.
/// - the Type/Subtype constellation.
///
///
/// A rule of thumb is this:
///
/// **Address 1:** \
/// The recipient station address.
/// If `to_ds` is set, this is the AP address.
/// If `from_ds` is set then this is the station address
///
/// **Address 2:** \
/// The transmitter station address.
/// If `from_ds` is set, this is the AP address.
/// If `to_ds` is set then this is the station address.
///
/// **Address 3:** \
/// If Address 1 contains the destination address then Address 3 will contain the source address.
/// Similarly, if Address 2 contains the source address then Address 3 will contain the destination address.
///
/// **Sequence Control:** \
/// Contains the FragmentNumber and SequenceNumber that define the main frame and the number of fragments in the frame.
///
/// **Address 4:** \
/// This is only set if a Wireless Distribution System (WDS) is being used (with AP to AP communication)
/// Address 1 contains the receiving AP address.
/// Address 2 contains the transmitting AP address.
/// Address 3 contains the destination station address.
/// Address 4 contains the source station address.
#[derive(Clone, Debug)]
pub struct Header {
    pub duration: [u8; 2],
    pub frame_control: FrameControl,
    pub address_1: MacAddress,
    pub address_2: MacAddress,
    pub address_3: MacAddress,
    pub address_4: Option<MacAddress>,
    pub seq_ctl: Option<[u8; 2]>,
}

impl Header {
    pub fn from_bytes(input: &[u8]) -> Result<(Header, &[u8])> {
        println!("Bytes: {:?}", &input);
        // Parse the frame control header. This is always present
        let frame_control = FrameControl::from_bytes(&input[0..2])?;
        println!(
            "Type/Subtype: {:?}, {:?}",
            frame_control.frame_type, frame_control.frame_subtype
        );

        // Read the duration. Bytes 2-3.
        // We don't do anything with this yet.
        let mut duration: [u8; 2] = [0; 2];
        duration.clone_from_slice(&input[2..4]);

        // Parse the first address, this is always exepected to be present.
        let address_1 = MacAddress::from_slice(&input[4..10]);

        let address_2 = MacAddress::from_slice(&input[10..16]);
        let address_3 = MacAddress::from_slice(&input[16..22]);

        // Read the sequence control bytes, it should always be present,
        // if the third address exists
        let mut seq_ctl: [u8; 2] = [0; 2];
        seq_ctl.clone_from_slice(&input[22..24]);
        let seq_ctl = Some(seq_ctl);

        // Whether the forth address exists, depends on two flags in FrameControl.
        // Depending on whether the address is set, the body begins at byte 24 or byte 30.
        let mut last_header_byte = 24;
        let mut address_4 = None;
        if frame_control.to_ds && frame_control.from_ds {
            address_4 = Some(MacAddress::from_slice(&input[24..30]));

            last_header_byte = 30;
        }

        let header = Header {
            duration,
            frame_control: frame_control.clone(),
            address_1,
            address_2,
            address_3,
            address_4,
            seq_ctl,
        };

        Ok((header, &input[last_header_byte..]))
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
            self.address_4.as_ref()
        }
    }
}
