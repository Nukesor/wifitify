use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Collector {
    /// If true, all channels will be checked once on startup.
    pub sweep_on_startup: bool,
    /// If true, all channels will always be checked.
    pub always_sweep: bool,
    /// If this is set to a channel, only this specific channel will be monitored.
    pub fixed_channel: Option<i32>,

    /// The time between full channel sweeps in seconds
    pub time_between_sweeps: i64,
    /// The time between channel switches during sweeps in milliseconds
    pub sweep_channel_switch_timeout: i64,
    /// The time between channel switches during normal mode in milliseconds
    pub channel_switch_timeout: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The TCP hostname/ip address.
    pub database_url: String,
    pub collector: Collector,
}

impl Config {
    /// Either get the config from an existing configuration file or
    /// create a new one from scratch
    pub fn new() -> Result<Self> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        if path.exists() {
            let mut file = File::open(path)?;
            let mut config = String::new();
            file.read_to_string(&mut config)?;

            let config: Config = toml::from_str(&config)?;
            return Ok(config);
        }

        // No config exists yet. Create a default config and persist it onto disk.
        let default_config = Config {
            database_url: "postgres://localhost/wifitify".into(),
            collector: Collector {
                sweep_on_startup: true,
                always_sweep: false,
                fixed_channel: None,
                time_between_sweeps: 7200,
                sweep_channel_switch_timeout: 5000,
                channel_switch_timeout: 250,
            },
        };
        default_config.write()?;

        Ok(default_config)
    }

    /// Write the current config to disk.
    pub fn write(&self) -> Result<()> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        let mut file = if path.exists() {
            File::open(path)?
        } else {
            File::create(path)?
        };

        let config = toml::to_string(&self)?;
        file.write_all(config.as_bytes())?;

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Couldn't find config dir")?;
        Ok(config_dir.join("wifitify.toml"))
    }
}
