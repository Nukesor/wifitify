use std::io;
use std::io::Read;

use anyhow::Result;
use bytes::{Buf, Bytes, IntoBuf};

use crate::frame_address::*;
use crate::frame_control::*;
use crate::info::*;

#[derive(Clone, Debug)]
pub struct Header {
    pub frame_control: FrameControl,
    pub duration: [u8; 2],
    pub dst: String,
    pub src: String,
    pub bssid: String,
    pub seq_ctl: [u8; 2],
    pub info: BodyInformation,
}

impl Header {
    pub fn from_bytes(input: &[u8]) -> Result<Header> {
        let buf = Bytes::from(input).into_buf();
        let mut reader = buf.reader();

        let mut control = [0; 2];
        reader.read(&mut control)?;
        let frame_control = FrameControl::from_bytes(&control)?;

        let mut duration = [0; 2];
        reader.read(&mut duration)?;

        let mut mac_addresses = [0; 18];
        reader.read(&mut mac_addresses)?;

        let (dst, src, bssid) = Header::parse_address(frame_control, &mac_addresses);

        let mut seq_ctl = [0; 2];
        reader.read(&mut seq_ctl)?;

        let mut dst2 = vec![];
        io::copy(&mut reader, &mut dst2)?;

        let body_information = Header::parse_body(frame_control, &dst2[..]);

        let header = Header {
            frame_control,
            duration,
            dst,
            src,
            bssid,
            seq_ctl,
            info: body_information,
        };
        Ok(header)
    }

    fn parse_address(frame_control: FrameControl, input: &[u8]) -> (String, String, String) {
        let mut dst = String::from("");
        let mut src = String::from("");
        let mut bssid = String::from("");

        let addresses = FrameAddresses::from_bytes(input).unwrap();

        if frame_control.to_ds && frame_control.from_ds {
            dst.push_str(&addresses.addr3.addr);
            src.push_str(&addresses.addr4.addr);
        } else if frame_control.to_ds {
            dst.push_str(&addresses.addr2.addr);
            src.push_str(&addresses.addr3.addr);
            bssid.push_str(&addresses.addr1.addr);
        } else if frame_control.from_ds {
            dst.push_str(&addresses.addr3.addr);
            src.push_str(&addresses.addr1.addr);
            bssid.push_str(&addresses.addr2.addr);
        } else {
            dst.push_str(&addresses.addr1.addr);
            src.push_str(&addresses.addr2.addr);
            bssid.push_str(&addresses.addr3.addr);
        }

        (dst, src, bssid)
    }

    fn parse_body(frame_control: FrameControl, input: &[u8]) -> BodyInformation {
        // For now, only management Frames are handled
        if !matches!(frame_control.frame_type, FrameType::Management) {
            return BodyInformation::UnHandled(true);
        }

        // Check which kind of frame sub-type we got
        match frame_control.frame_subtype {
            FrameSubType::Beacon => BodyInformation::Beacon(Beacon::from_bytes(input)),
            FrameSubType::ProbeReq => {
                BodyInformation::ProbeRequest(ProbeRequest::from_bytes(input))
            }
            FrameSubType::ProbeResp => {
                BodyInformation::ProbeResponse(ProbeResponse::from_bytes(input))
            }
            FrameSubType::AssoReq => {
                BodyInformation::AssociationRequest(AssociationRequest::from_bytes(input))
            }
            FrameSubType::AssoResp => {
                BodyInformation::AssociationResponse(AssociationResponse::from_bytes(input))
            }
            _ => BodyInformation::UnHandled(true),
        }
    }
}
