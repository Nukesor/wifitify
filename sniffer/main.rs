use anyhow::{bail, Context, Result};
use clap::Clap;
use iee_802_11::*;
use log::{debug, error, info};
use pcap::Device;
use radiotap::Radiotap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;

use cli::CliArguments;

fn main() -> Result<()> {
    better_panic::install();
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

    // Set pcap Datalink type to IEE 802.11
    // http://www.tcpdump.org/linktypes.html
    // DLT_IEEE802_11_RADIO = 127
    capture
        .set_datalink(pcap::Linktype(127))
        .context("Failed to set wifi datalink type")?;

    while let Ok(packet) = capture.next() {
        // At first, we look at the
        let radiotap = match Radiotap::from_bytes(packet.data) {
            Ok(radiotap) => radiotap,
            Err(error) => {
                error!(
                    "Couldn't read packet data with Radiotap: {:?}, error {:?}",
                    &packet.data, error
                );
                continue;
            }
        };

        let payload = &packet.data[radiotap.header.length..];
        if let Err(err) = handle_ieee_802_11_payload(payload) {
            debug!("Error during frame handling:\n{:?}", err);
        };
    }

    Ok(())
}

fn handle_ieee_802_11_payload(bytes: &[u8]) -> Result<()> {
    let frame = Frame::parse(bytes)?;
    //if let Some(ap) = mapper.map(tap_data, dot11_header, people) {
    //    term.write_line(&format!(
    //        "Access point {} signal {} current channel {} {}",
    //        style(ap.ssid).cyan(),
    //        style(ap.signal).cyan(),
    //        style(ap.current_channel).cyan(),
    //        "                      "
    //    ))?;
    //}

    let dest = frame
        .header
        .dest()
    if frame.header.src().is_none() {
        info!(
            "Got type {:?} ({:?}) to {}",
            frame.header.frame_control.frame_type,
            frame.header.frame_control.frame_subtype,
            dest.to_string()
        );
        return Ok(());
    }
    let src = frame.header.src().unwrap();

    info!(
        "Type {:?} ({:?}) from {} to {}",
        frame.header.frame_control.frame_type,
        frame.header.frame_control.frame_subtype,
        src.to_string(),
        dest.to_string()
    );

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
