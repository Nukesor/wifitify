use anyhow::Result;
use clap::Clap;
use db::{
    models::{Data, Station},
    DbPool,
};
use libwifi::{Addresses, Frame, HasHeader};
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod db;
mod wifi;

use crate::cli::CliArguments;
use crate::wifi::capture::*;

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

    loop {
        let frame = receiver.recv()?;

        extract_data(frame, &pool).await?;
    }
}

async fn extract_data(frame: Frame, pool: &DbPool) -> Result<()> {
    match frame {
        Frame::Beacon(frame) => {
            let station = Station {
                id: 0,
                mac_address: frame.src().unwrap().clone(),
                ssid: frame.station_info.ssid.clone(),
                nickname: None,
                description: None,
            };

            Station::persist(&station, &pool).await?;
        }
        _ => println!("Ignoring frame: {:?}", frame),
    };

    Ok(())
}
