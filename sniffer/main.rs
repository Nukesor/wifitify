use std::io::Cursor;

use anyhow::{bail, Context, Result};
use bytes::Buf;
use clap::Clap;
use iee_800_11::header::Header;
use log::{debug, error, info};
use pcap::{Device, Packet};
use radiotap::Radiotap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;

use cli::CliArguments;

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    SimpleLogger::init(level, Config::default()).unwrap();

    let device = find_device_by_name(&opt.device)?;

    let mut capture = device.open().context("Failed to open device")?;

    // Set pcap Datalink type to IEE 800.11
    // http://www.tcpdump.org/linktypes.html
    // DLT_IEEE802_11_RADIO = 127
    capture
        .set_datalink(pcap::Linktype(127))
        .context("Failed to set wifi datalink type")?;

    while let Ok(packet) = capture.next() {
        if let Err(err) = handle_packet(packet) {
            error!("Got error handling packet: {:?}", err);
        }
    }

    Ok(())
}

fn handle_packet(packet: Packet) -> Result<()> {
    debug!("received packet! {:?}", packet);
    let radiotap_data = match Radiotap::from_bytes(packet.data) {
        Ok(radiotap) => radiotap,
        Err(error) => bail!(
            "Couldn't read packet data with Radiotap: {:?}, error {:?}",
            &packet.data,
            error
        ),
    };

    let mut packet_cursor = Cursor::new(packet.data);
    packet_cursor.advance(radiotap_data.header.length);

    let header = Header::from_bytes(&packet_cursor.bytes())?;
    //if let Some(ap) = mapper.map(tap_data, dot11_header, people) {
    //    term.write_line(&format!(
    //        "Access point {} signal {} current channel {} {}",
    //        style(ap.ssid).cyan(),
    //        style(ap.signal).cyan(),
    //        style(ap.current_channel).cyan(),
    //        "                      "
    //    ))?;
    //}

    println!("");

    Ok(())
}

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
