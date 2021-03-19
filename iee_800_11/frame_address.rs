use anyhow::Result;
use bytes::{Buf, Bytes, IntoBuf};

#[derive(Clone, Debug)]
pub struct MACField {
    pub addr: String,
}

impl MACField {
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
