use anyhow::{bail, Context, Result};
use clap::Clap;
use ieee80211::{Frame, FrameLayer};
use log::{debug, error, info, warn};
use pcap::Device;
use radiotap::Radiotap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;

use crate::cli::CliArguments;

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

    let device = find_device_by_name("wlp4s0")?;

    let mut capture = device.open().context("Failed to open device")?;

    while let Ok(packet) = capture.next() {
        debug!("received packet! {:?}", packet);
        let radiotap = if let Ok(radiotap) = Radiotap::from_bytes(packet.data) {
            radiotap
        } else {
            warn!(
                "Couldn't read packet data with Radiotap: {:?}",
                &packet.data
            );
            continue;
        };

        info!("");
    }

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
