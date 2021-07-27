use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use log::info;

use wifitify::config::Config;
use wifitify::db::models::*;
use wifitify::db::DbPool;

pub struct AppState {
    /// The current configuration
    pub config: Config,

    /// Our local cache for the station database table.
    pub stations: HashMap<String, Station>,
    /// Our local cache for the device database table.
    pub devices: HashMap<String, Device>,
    pub station_device_map: HashMap<i32, HashSet<i32>>,

    /// The list of channels that are currently being scanned.
    pub watched_channels: Vec<i32>,
    /// Since the list of watched channels updates we cannot create a long-running iterator
    /// over the watched_channel list (iter borrows the Vec).
    /// That's why we have to do this manually.
    pub current_watched_channels: i32,

    /// last_channel_switch: The last time we switched a channel during full sweep.
    pub last_channel_switch: DateTime<Utc>,
    /// last_full_sweep: The last time checked all channels for new stations.
    pub last_full_sweep: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        // Set the last channel sweep and switch to the past.
        // That way we start with a sweep right away.
        let last_full_sweep = Utc::now();
        let last_channel_switch = Utc::now()
            .checked_sub_signed(Duration::hours(2))
            .expect("This should happen.");

        let mut state = AppState {
            config: Config::new()?,

            stations: HashMap::new(),
            devices: HashMap::new(),
            station_device_map: HashMap::new(),
            watched_channels: Vec::new(),
            current_watched_channels: 0,
            last_full_sweep,
            last_channel_switch,
        };

        // If the user wants to always sweep on the first run, immediately schedule a sweep.
        if state.config.collector.sweep_on_startup {
            state.schedule_sweep();
        }

        Ok(state)
    }

    pub async fn init_state(
        &mut self,
        pool: &mut DbPool,
        supported_channels: &[i32],
    ) -> Result<()> {
        let mut connection = pool
            .acquire()
            .await
            .expect("Couldn't get connection during state initialization");

        // Initialize database cache for known stations and devices.
        self.stations = Station::known_stations(&mut connection).await?;
        self.devices = Device::known_devices(&mut connection).await?;
        self.station_device_map = DeviceStation::get_station_device_map(&mut connection).await?;
        self.update_watched_channels(supported_channels);
        info!("Watching channels: {:?}", self.watched_channels);

        Ok(())
    }

    /// Returns, whether it's time to do the next full sweep.
    pub fn should_sweep(&self) -> bool {
        (Utc::now() - self.last_full_sweep)
            > Duration::seconds(self.config.collector.time_between_sweeps)
    }

    /// If this is called, a new full sweep will be started in the next iteration step.
    pub fn schedule_sweep(&mut self) {
        self.last_full_sweep = Utc::now()
            .checked_sub_signed(Duration::hours(2))
            .expect("This shouldn't happen.");
    }

    /// Convenience method to check, whether it's time to switch to the next channel.
    pub fn should_switch_channel(&self) -> bool {
        if self.should_sweep() {
            (Utc::now() - self.last_channel_switch)
                > Duration::milliseconds(self.config.collector.sweep_channel_switch_timeout)
        } else {
            (Utc::now() - self.last_channel_switch)
                > Duration::milliseconds(self.config.collector.channel_switch_timeout)
        }
    }

    /// Update the list of unqiue channels that are used by any of the watched stations.
    pub fn update_watched_channels(&mut self, supported_channels: &[i32]) {
        self.watched_channels = self
            .stations
            .values()
            .filter(|station| station.watch && supported_channels.contains(&station.channel))
            .map(|station| station.channel)
            .collect::<Vec<i32>>();

        self.watched_channels.sort_unstable();
        self.watched_channels.dedup();

        if self.watched_channels.is_empty() {
            info!("No active stations. Starting next full sweep");
            self.schedule_sweep();
        }
    }

    /// Get the next entry in our list of watched channels.
    /// If we're at the end or if the list is empty, `None` will be returned
    pub fn get_next_watched_channel(&mut self) -> Option<i32> {
        // Get the fixed channel if it's set
        if let Some(channel) = self.config.collector.fixed_channel {
            return Some(channel);
        }

        // Reschedule a full sweep, in case there aren't any watched cannels.
        if self.watched_channels.is_empty() {
            info!("No active stations. Starting next full sweep");
            self.schedule_sweep();

            return None;
        }

        self.current_watched_channels += 1;
        if self.current_watched_channels as usize >= self.watched_channels.len() {
            self.current_watched_channels = 0;
        }

        Some(self.watched_channels[self.current_watched_channels as usize])
    }
}
