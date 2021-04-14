use anyhow::Result;
use clap::Clap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod db;
mod wifi;

use crate::cli::CliArguments;
use crate::wifi::capture::*;
use crate::wifi::device::*;

#[async_std::main]
async fn main() -> Result<()> {
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

    let mut capture = get_capture(&opt.device)?;

    let pool = db::init_pool().await?;

    while let Ok(packet) = capture.next() {
        handle_packet(packet)?;
    }

    Ok(())
}
