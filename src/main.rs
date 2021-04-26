use std::collections::HashSet;

use anyhow::{bail, Result};
use chrono::{Timelike, Utc};
use clap::Clap;
use crossbeam_channel::{unbounded, RecvTimeoutError};
use libwifi::frame::components::MacAddress;
use libwifi::{Addresses, Frame};
use log::{debug, info, warn, LevelFilter};
use pretty_env_logger::formatted_builder;
use radiotap::Radiotap;

mod cli;
mod db;
mod device;
mod state;
mod wifi;

use crate::cli::CliArguments;
use crate::wifi::capture::*;
use db::models::*;
use db::DbPool;
use device::{get_mhz_to_channel, supported_channels, switch_channel};
use state::AppState;

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
    let mut builder = formatted_builder();
    builder
        .filter(None, level)
        .filter(Some("sqlx::query"), LevelFilter::Error)
        .init();

    // Initialize the database connection pool
    let pool: DbPool = db::init_pool().await?;

    // The channel to send Wifi frames from the receiver thread
    let (sender, receiver) = unbounded::<(Frame, Radiotap)>();

    // The data capture and parsing logic is running in its own thread.
    // This allows us to have all receiving logic in a non-blocking fashion.
    // The actual handling of the received frames can then be done in an async fashion, since
    // there'll be a lot of I/O wait when interacting with the database.
    let mut capture = get_capture(&opt.device)?;
    let supported_channels = supported_channels(&opt.device)?;
    info!("Found supported channels: {:?}", supported_channels);

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

    let mut state = AppState::new();
    // Initialize database cache for known stations and devices.
    state.stations = Station::known_stations(&pool).await?;
    state.devices = Device::known_devices(&pool).await?;
    state.station_device_map = DeviceStation::get_station_device_map(&pool).await?;
    state.update_watched_channels();
    info!("Watching channels: {:?}", state.watched_channels);

    // Channel iterator that's used to walk through all channels
    let mut supported_channel_iter = supported_channels.iter();

    loop {
        let doing_sweep = state.should_sweep();

        // Try to receive for a few milliseconds.
        // Sometimes we might walk over channels that don't have any active devices.
        // If we would keep listening on those devices, we would be wait forever!
        match receiver.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok((frame, radiotap)) => {
                extract_data(&pool, &mut state, frame, radiotap, doing_sweep).await?
            }
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => {
                bail!("The mpsc channel to the receiver thread disconnected.")
            }
        }

        // Check whether we're currently doing a full sweep.
        // If we aren't, cycle through all watched channels.
        if !doing_sweep {
            if state.should_switch_channel() {
                let next_channel = state.get_next_watched_channel();
                switch_channel(&opt.device, next_channel)?;
                debug!("Switching to channel {}", next_channel);
                state.last_channel_switch = Utc::now();
            }

            continue;
        }
        // Check whether we should switch the channel right now. Otherwise, just continue.
        if !state.should_switch_channel() {
            continue;
        }

        // Check if there's another channel we should check.
        // If that's not the case, we set the last full sweep and continue.
        let next_channel = if let Some(next_channel) = supported_channel_iter.next() {
            *next_channel
        } else {
            state.last_full_sweep = Utc::now();
            info!("Full sweep finished");
            supported_channel_iter = supported_channels.iter();
            state.update_watched_channels();
            info!("Watched channels are: {:?}", &state.watched_channels);
            continue;
        };

        switch_channel(&opt.device, next_channel)?;
        debug!("Switching to channel {}", next_channel);
        state.last_channel_switch = Utc::now();
    }
}

async fn extract_data(
    pool: &DbPool,
    state: &mut AppState,
    frame: Frame,
    radiotap: Radiotap,
    should_update: bool,
) -> Result<()> {
    match frame {
        Frame::Beacon(frame) => {
            let station_mac = frame.src().unwrap().clone();
            let station_mac_string = station_mac.to_string();

            // Ignore the packet, if we cannot get the channel.
            let channel_mhz = if let Some(channel) = radiotap.channel {
                channel.freq
            } else {
                return Ok(());
            };

            let channel = if let Some(channel) = get_mhz_to_channel(channel_mhz) {
                channel
            } else {
                warn!(
                    "Couldn't find channel for unknown frequency {}MHz.",
                    channel_mhz
                );
                return Ok(());
            };

            // We already know this station
            // In case we're doing a full hannel sweep right now, update any station metadata.
            if let Some(station) = state.stations.get_mut(&station_mac_string) {
                if should_update {
                    station.channel = channel;
                    station.ssid = frame.station_info.ssid.clone();
                    station.power_level = radiotap.antenna_signal.map(|a| a.value as i32);
                    station.update_metadata(pool).await?;
                }
                return Ok(());
            }
            // Add the station to the database
            let mut station = Station {
                id: 0,
                mac_address: station_mac.into(),
                ssid: frame.station_info.ssid.clone(),
                channel,
                power_level: radiotap.antenna_signal.map(|a| a.value as i32),
                nickname: None,
                description: None,
                watch: false,
            };
            station.id = Station::persist(&station, &pool).await?;

            info!(
                "Found station {} with ssid: {:?}",
                station_mac_string,
                frame.station_info.ssid.clone()
            );

            state.stations.insert(station_mac_string, station);
        }
        Frame::Data(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(pool, state, src, dest, frame.data.len() as i32).await?;
        }
        Frame::QosData(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(pool, state, src, dest, frame.data.len() as i32).await?;
        }
        _ => (), // println!("Ignoring frame: {:?}", frame),
    };

    Ok(())
}

async fn log_data_frame(
    pool: &DbPool,
    state: &mut AppState,
    src: &MacAddress,
    dest: &MacAddress,
    data_length: i32,
) -> Result<()> {
    // Data frames can go in both directions.
    // Check if either src or dest is a known station, the other one has to be the device.
    // If none is a known station, we just return.
    let (station, device_mac) = if let Some(id) = state.stations.get(&src.to_string()) {
        (id, dest)
    } else if let Some(id) = state.stations.get(&dest.to_string()) {
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
    let device = if let Some(device) = state.devices.get(&device_mac.to_string()) {
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
        state
            .devices
            .entry(device_mac.to_string())
            .or_insert(device)
    };

    // Only track activity on explitly watched stations and devices.
    if !(station.watch && device.watch) {
        return Ok(());
    }

    let mut time = Utc::now();
    time = time.with_second(0).unwrap();
    time = time.with_nanosecond(0).unwrap();

    debug!(
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

    // Register the relationship between device and station, if it's new.
    let set = state
        .station_device_map
        .entry(station.id)
        .or_insert(HashSet::new());
    if !set.contains(&device.id) {
        let device_station = DeviceStation {
            station: station.id,
            device: device.id,
        };
        device_station.persist(pool).await?;
        set.insert(device.id);
    }

    Ok(())
}
