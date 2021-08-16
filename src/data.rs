use anyhow::Result;
use chrono::{Timelike, Utc};
use libwifi::frame::components::MacAddress;
use libwifi::frame::BlockAckInfo;
use libwifi::{Addresses, Frame};
use log::{debug, info, warn};
use radiotap::Radiotap;

use crate::db::models::*;
use crate::db::{Connection, DbPool};
use crate::device::get_mhz_to_channel;

pub async fn handle_packet(pool: DbPool, frame: Frame, radiotap: Radiotap, doing_sweep: bool) {
    let tries: i32 = 3;
    let mut current_try: i32 = 0;

    // We try to get a connection sevaral times and wait a short amount between each try.
    // Afterwards we fail hard.
    let connection = loop {
        let result = pool.acquire().await;

        // We didn't get a connection, retry in a sec or fail hard on limit.
        if result.is_err() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            current_try += 1;
            if current_try == tries {
                break None;
            }

            continue;
        }

        break Some(result.unwrap());
    };

    if connection.is_none() {
        println!("Failed to get connection from pool after three seconds");
        return;
    }
    let mut connection = connection.unwrap();

    let result = extract_data(&mut connection, frame, radiotap, doing_sweep).await;
    if let Err(err) = result {
        println!("Got error while handling packet {:?}", err)
    };
}

async fn extract_data(
    connection: &mut Connection,
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
            if let Some(mut station) = Station::get_by_mac(connection, &station_mac_string).await? {
                if should_update {
                    station.channel = channel;
                    station.ssid = frame.station_info.ssid.clone();
                    station.power_level = radiotap.antenna_signal.map(|a| a.value as i32);
                    station.update_metadata(connection).await?;
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
            station.persist(connection).await?;

            info!(
                "Found station {} with ssid: {:?}",
                station_mac_string,
                frame.station_info.ssid.clone()
            );
        }
        Frame::Data(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(connection, src, dest, frame.data.len() as i32).await?;
        }
        Frame::QosData(frame) => {
            let src = frame.src().expect("Data frames always have a source");
            let dest = frame.dest();

            log_data_frame(connection, src, dest, frame.data.len() as i32).await?;
        }
        Frame::BlockAck(frame) => {
            let src = frame
                .src()
                .expect("BlockAck frames always have a source")
                .clone();
            let dest = frame.dest().clone();

            match frame.acks {
                BlockAckInfo::Basic(_) => {
                    log_data_frame(connection, &src, &dest, 100).await?;
                }
                BlockAckInfo::Compressed(acks) => {
                    log_data_frame(connection, &src, &dest, (acks.len() * 500) as i32).await?;
                }
            }
        }
        _ => (), // println!("Ignoring frame: {:?}", frame),
    };

    Ok(())
}

async fn log_data_frame(
    connection: &mut Connection,
    src: &MacAddress,
    dest: &MacAddress,
    data_length: i32,
) -> Result<()> {
    // Data frames can go in both directions.
    // Check if either src or dest is a known station, the other one has to be the device.
    // If none is a known station, we just return.
    let (station, device_mac) =
        if let Some(station) = Station::get_by_mac(connection, &src.to_string()).await? {
            (station, dest)
        } else if let Some(station) = Station::get_by_mac(connection, &dest.to_string()).await? {
            (station, src)
        } else {
            return Ok(());
        };

    // Ignore multicasts/broadcasts and other meta stuff.
    if !device_mac.is_real_device() {
        return Ok(());
    }

    // Either get the device id from the known device map.
    // If it's not in there yet, register a new client and add the client id to the map.
    let device =
        if let Some(device) = Device::get_by_mac(connection, &device_mac.to_string()).await? {
            device
        } else {
            let mut device = Device {
                id: 0,
                mac_address: device_mac.clone().into(),
                nickname: None,
                description: None,
                watch: true,
            };

            device.id = device.persist(connection).await?;
            device
        };

    // Only track activity on explitly watched stations and devices.
    if !(station.watch && device.watch) {
        return Ok(());
    }

    let mut time = Utc::now();
    time = time.with_second(0).unwrap();
    time = time.with_nanosecond(0).unwrap();

    let device_name = device
        .nickname
        .clone()
        .unwrap_or_else(|| device_mac.to_string());
    debug!(
        "Got {} bytes data from/to device {}",
        data_length, device_name
    );

    let data = Data {
        time,
        device: device.id,
        station: station.id,
        bytes_per_minute: data_length,
    };

    data.persist(connection).await?;

    // Register the relationship between device and station, if it's new.
    if DeviceStation::get_by_station_device(connection, station.id, device.id)
        .await?
        .is_none()
    {
        let device_station = DeviceStation {
            station: station.id,
            device: device.id,
        };
        device_station.persist(connection).await?;
    }

    Ok(())
}
