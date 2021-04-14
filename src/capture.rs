use anyhow::Result;
use libwifi::components::MacAddress;
use libwifi::frame::Frame;
use libwifi::frame_types::FrameSubType;
use libwifi::traits::*;
use log::{debug, error, info};
use pcap::Packet;
use radiotap::Radiotap;

pub fn handle_packet(packet: Packet) -> Result<()> {
    // At first, we look at the
    let radiotap = match Radiotap::from_bytes(packet.data) {
        Ok(radiotap) => radiotap,
        Err(error) => {
            error!(
                "Couldn't read packet data with Radiotap: {:?}, error {:?}",
                &packet.data, error
            );
            return Ok(());
        }
    };

    let payload = &packet.data[radiotap.header.length..];
    debug!("Full packet: {:?}", payload);
    if let Err(err) = handle_ieee_802_11_payload(payload) {
        debug!("Error during frame handling:\n{}", err);
        match err {
            libwifi::error::Error::Failure(_, data) => debug!("{:?}", data),
            _ => (),
        }
    };

    Ok(())
}

#[derive(Clone, Debug)]
struct ExtractedData {
    pub frame_type: FrameSubType,
    pub data: Option<i32>,
    pub src: Option<MacAddress>,
    pub dest: MacAddress,
    pub ssid: Option<String>,
}

fn handle_ieee_802_11_payload(bytes: &[u8]) -> Result<(), libwifi::error::Error> {
    let frame = libwifi::parse(bytes)?;

    let data = match frame {
        Frame::Beacon(frame) => ExtractedData {
            src: frame.src().map(Clone::clone),
            dest: frame.dest().clone(),
            ssid: frame.station_info.ssid.clone(),
            data: None,
            frame_type: FrameSubType::Beacon,
        },
        Frame::ProbeRequest(frame) => ExtractedData {
            src: frame.src().map(Clone::clone),
            dest: frame.dest().clone(),
            ssid: None,
            data: None,
            frame_type: FrameSubType::ProbeRequest,
        },
        Frame::ProbeResponse(frame) => ExtractedData {
            src: frame.src().map(Clone::clone),
            dest: frame.dest().clone(),
            ssid: frame.station_info.ssid.clone(),
            data: None,
            frame_type: FrameSubType::ProbeResponse,
        },
        _ => return Ok(()),
    };

    debug!("Extracted data: {:?}", data);

    Ok(())
}
