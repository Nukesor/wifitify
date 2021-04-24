use std::collections::HashMap;

use anyhow::Result;
use chrono::Timelike;
use clap::Clap;
use libwifi::frame::components::MacAddress;
use libwifi::{Addresses, Frame};
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod db;
mod wifi;

use crate::cli::CliArguments;
use crate::wifi::capture::*;
use db::models::{Data, Device, Station};
use db::DbPool;

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
    let pool = db::init_pool().await?;

    // The channel to send Wifi frames from the receiver thread
    let (sender, receiver) = std::sync::mpsc::channel::<Frame>();

    // The data capture and parsing logic is running in its own thread.
    // This allows us to have all receiving logic in a non-blocking fashion.
    // The actual handling of the received frames can then be done in an async fashion, since
    // there'll be a lot of I/O wait when interacting with the database.
    let mut capture = get_capture(&opt.device)?;
    std::thread::spawn(move || {
        while let Ok(packet) = capture.next() {
            if let Ok(data) = handle_packet(packet) {
                // Send extracted data to the receiver.
                // This only errors if the receiver went away, in which case we just bail.
                if let Err(_) = sender.send(data) {
                    return;
                };
            }
        }
    });

    let mut stations = Station::known_macs(&pool).await?;
    let mut devices = Device::known_macs(&pool).await?;

    println!("Outside: {:?}", &stations);
    loop {
        let frame = receiver.recv()?;

        extract_data(frame, &pool, &mut stations, &mut devices).await?;
    }
}

async fn extract_data(
    frame: Frame,
    pool: &DbPool,
    stations: &mut HashMap<String, i32>,
    devices: &mut HashMap<String, i32>,
) -> Result<()> {
    match frame {
        Frame::Beacon(frame) => {
            let station_mac = frame.src().unwrap().clone();
            let station_mac_string = station_mac.to_string();

            println!("Got station {:?}", frame.station_info.ssid.clone());

            // We already know this station
            if stations.contains_key(&station_mac_string) {
                return Ok(());
            }
            let station = Station {
                id: 0,
                mac_address: station_mac.into(),
                ssid: frame.station_info.ssid.clone(),
                nickname: None,
                description: None,
            };

            let id = Station::persist(&station, &pool).await?;

            stations.insert(station_mac_string, id);
        }
        Frame::Data(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            handle_data_frame(pool, src, dest, frame.data.len() as i32, stations, devices).await?;
        }
        Frame::QosData(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            handle_data_frame(pool, src, dest, frame.data.len() as i32, stations, devices).await?;
        }
        _ => println!("Ignoring frame: {:?}", frame),
    };

    Ok(())
}

async fn handle_data_frame(
    pool: &DbPool,
    src: &MacAddress,
    dest: &MacAddress,
    data_length: i32,
    stations: &mut HashMap<String, i32>,
    devices: &mut HashMap<String, i32>,
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

    // Ignore Ipv6 multicasts
    if device_mac.is_ipv6_multicast() {
        return Ok(());
    }

    // Either get the device id from the known device map.
    // If it's not in there yet, register a new client and add the client id to the map.
    let device = if let Some(id) = devices.get(&device_mac.to_string()) {
        *id
    } else {
        let device = Device {
            id: 0,
            mac_address: device_mac.clone().into(),
            nickname: None,
            description: None,
            station: *station,
        };

        let id = device.persist(&pool).await?;
        devices.insert(device_mac.to_string(), id);

        id
    };

    let mut time = chrono::offset::Local::now().naive_local();
    time = time.with_second(0).unwrap();
    time = time.with_nanosecond(0).unwrap();

    let data = Data {
        time,
        device,
        station: *station,
        bytes_per_minute: data_length,
    };

    data.persist(pool).await?;

    Ok(())
}
