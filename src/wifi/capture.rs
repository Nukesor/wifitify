use anyhow::{bail, Context, Result};
use log::info;
use pcap::{Active, Capture, Device};

use libwifi::*;
use pcap::Packet;
use radiotap::Radiotap;

/// Parse the packet received by [pcap](::pcap)
pub fn handle_packet(packet: Packet) -> Result<(Frame, Radiotap)> {
    // Read the raw payload, which
    let radiotap = Radiotap::from_bytes(packet.data)?;

    let bytes = &packet.data[radiotap.header.length..];
    let frame = libwifi::parse_frame(bytes)?;

    Ok((frame, radiotap))
}

/// Initializes and configures a network device by name.
/// The continuous capture stream of the device is returned.
pub fn get_capture(device_name: &str) -> Result<Capture<Active>> {
    let device = find_device_by_name(device_name)?;
    let capture = Capture::from_device(device)?.immediate_mode(true);

    let mut capture = capture
        .open()
        .context("Failed to open capture on device.")?;

    // Set pcap Datalink type to IEE 802.11
    // http://www.tcpdump.org/linktypes.html
    // DLT_IEEE802_11_RADIO = 127
    capture
        .set_datalink(pcap::Linktype(127))
        .context("Failed to set wifi datalink type")?;

    Ok(capture)
}

/// Check if a device with a given name exists.
/// If that's the case, return it.
fn find_device_by_name(name: &str) -> Result<Device> {
    let devices = Device::list().context("Failed during device lookup:")?;
    for device in devices {
        info!("Found device {:?}", device.name);
        if device.name == name {
            return Ok(device);
        }
    }

    bail!("Couldn't find device with name {}", name)
}
