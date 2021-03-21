use anyhow::Result;
use bytes::{Buf, Bytes, IntoBuf};

/// This is our representation of a MAC-address
#[derive(Clone, Debug)]
pub struct MACField {
    pub addr: String,
}

impl MACField {
    /// Get the mac address from a 6 byte slice.
    pub fn from_slice(s: &[u8]) -> MACField {
        let addr = format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            s[0], s[1], s[2], s[3], s[4], s[5]
        );

        MACField { addr }
    }
}

#[derive(Clone, Debug)]
pub struct FrameAddresses {
    pub addr1: MACField,
    pub addr2: MACField,
    pub addr3: MACField,
    pub addr4: MACField,
}

/// Bytes 4-29 contain all important address information.
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
/// Address 1:
/// The transmitter station address on the BSS.
/// If `from_ds` is set, this is the AP address.
/// If `to_ds` is set then this is the station address.
///
/// Address 3:
/// If Address 1 contains the destination address then Address 3 will contain the source address.
/// Similarly, if Address 2 contains the source address then Address 3 will contain the destination address.
///
/// Address 4:
/// This is only set if a Wireless Distribution System (WDS) is being used (with AP to AP communication)
/// Address 1 contains the receiving AP address.
/// Address 2 contains the transmitting AP address.
/// Address 3 contains the destination station address.
/// Address 4 contains the source station address.
///
/// Sequence Control:
/// Contains the FragmentNumber and SequenceNumber that define the main frame and the number of fragments in the frame.
///
impl FrameAddresses {
    pub fn from_bytes(s: &[u8]) -> Result<FrameAddresses> {
        use std::io::Read;

        let buf = Bytes::from(s).into_buf();
        let mut reader = buf.reader();

        let mut mac_addr1 = [0; 6];
        reader.read(&mut mac_addr1)?;
        let addr1 = MACField::from_slice(&mac_addr1);

        let mut mac_addr2 = [0; 6];
        reader.read(&mut mac_addr2)?;
        let addr2 = MACField::from_slice(&mac_addr2);

        let mut mac_addr3 = [0; 6];
        reader.read(&mut mac_addr3)?;
        let addr3 = MACField::from_slice(&mac_addr3);

        let mut seq_ctl = [0; 2];
        reader.read(&mut seq_ctl)?;

        let mut mac_addr4 = [0; 6];
        reader.read(&mut mac_addr4)?;
        let addr4 = MACField::from_slice(&mac_addr4);

        Ok(FrameAddresses {
            addr1,
            addr2,
            addr3,
            addr4,
        })
    }
}
