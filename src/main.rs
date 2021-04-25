use std::collections::HashMap;

use anyhow::Result;
use chrono::Timelike;
use clap::Clap;
use libwifi::frame::components::MacAddress;
use libwifi::{Addresses, Frame};
use radiotap::Radiotap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod db;
mod device;
mod wifi;

use crate::cli::CliArguments;
use crate::wifi::capture::*;
use db::models::{Data, Device, Station};
use db::DbPool;
use device::{get_mhz_to_channel, supported_channels};

#[async_std::main]
async fn main() -> Result<()> {
    // Beautify panics for better debug output.
    better_panic::install();

    // Parse commandline options.
    let opt = CliArguments::parse();

    // Set the verbosity level and initialize the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    SimpleLogger::init(level, Config::default()).unwrap();

    // Initialize the database connection pool
    let pool: DbPool = db::init_pool().await?;

    let mut capture = get_capture(&opt.device)?;
    let supported_channels = supported_channels(&opt.device)?;
    println!("Found supported channels: {:?}", supported_channels);

    // Cache for known stations and devices.
    let mut stations = Station::known_stations(&pool).await?;
    let mut devices = Device::known_devices(&pool).await?;

    while let Ok(packet) = capture.next() {
        if let Ok((frame, radiotap)) = handle_packet(packet) {
            extract_data(frame, &pool, &mut stations, &mut devices, &radiotap).await?;
        }
    }

    Ok(())
}

async fn extract_data(
    frame: Frame,
    pool: &DbPool,
    stations: &mut HashMap<String, Station>,
    devices: &mut HashMap<String, Device>,
    radiotap: &Radiotap,
) -> Result<()> {
    match frame {
        Frame::Beacon(frame) => {
            let station_mac = frame.src().unwrap().clone();
            let station_mac_string = station_mac.to_string();

            //println!("Got station {:?}", frame.station_info.ssid.clone());

            // We already know this station
            if stations.contains_key(&station_mac_string) {
                return Ok(());
            }

            // Ignore the packet, if we cannot get the channel.
            let channel_mhz = if let Some(channel) = radiotap.channel {
                channel.freq
            } else {
                return Ok(());
            };

            let channel = if let Some(channel) = get_mhz_to_channel(channel_mhz) {
                channel
            } else {
                println!(
                    "Couldn't find channel for unknown frequency {}MHz.",
                    channel_mhz
                );
                return Ok(());
            };

            // Add the station to the database
            let mut station = Station {
                id: 0,
                mac_address: station_mac.into(),
                ssid: frame.station_info.ssid.clone(),
                nickname: None,
                description: None,
                watch: false,
                channel,
            };
            station.id = Station::persist(&station, &pool).await?;

            stations.insert(station_mac_string, station);
        }
        Frame::Data(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(pool, src, dest, frame.data.len() as i32, stations, devices).await?;
        }
        Frame::QosData(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(pool, src, dest, frame.data.len() as i32, stations, devices).await?;
        }
        _ => (), // println!("Ignoring frame: {:?}", frame),
    };

    Ok(())
}

async fn log_data_frame(
    pool: &DbPool,
    src: &MacAddress,
    dest: &MacAddress,
    data_length: i32,
    stations: &mut HashMap<String, Station>,
    devices: &mut HashMap<String, Device>,
) -> Result<()> {
    // Data frames can go in both directions.
    // Check if either src or dest is a known station, the other one has to be the device.
    // If none is a known station, we just return.
    let (station, device_mac) = if let Some(id) = stations.get(&src.to_string()) {
        (id, dest)
    } else if let Some(id) = stations.get(&dest.to_string()) {
        (id, src)
    } else {
        return Ok(());
    };

    // Ignore multicasts/broadcasts and other meta stuff.
    if !device_mac.is_real_device() {
        return Ok(());
    }

    // Either get the device id from the known device map.
    // If it's not in there yet, register a new client and add the client id to the map.
    let device = if let Some(device) = devices.get(&device_mac.to_string()) {
        device
    } else {
        let mut device = Device {
            id: 0,
            mac_address: device_mac.clone().into(),
            nickname: None,
            description: None,
            watch: true,
        };

        device.id = device.persist(&pool).await?;
        devices.entry(device_mac.to_string()).or_insert(device)
    };

    // Only track activity on explitly watched stations and devices.
    if !(station.watch && device.watch) {
        return Ok(());
    }

    let mut time = chrono::offset::Local::now().naive_local();
    time = time.with_second(0).unwrap();
    time = time.with_nanosecond(0).unwrap();

    println!(
        "Got {} bytes data from/to device {}",
        data_length,
        device_mac.to_string()
    );

    let data = Data {
        time,
        device: device.id,
        station: station.id,
        bytes_per_minute: data_length,
    };

    data.persist(pool).await?;

    Ok(())
}
