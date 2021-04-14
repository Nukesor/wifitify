use anyhow::Result;
use clap::Clap;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod capture;
mod cli;
mod device;

use capture::*;
use cli::CliArguments;
use device::*;

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

    let mut capture = get_capture(&opt.device)?;

    while let Ok(packet) = capture.next() {
        handle_packet(packet)?;
    }

    Ok(())
}
