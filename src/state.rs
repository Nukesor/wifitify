use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use crate::db::models::{Device, Station};

pub struct AppState {
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

    /// The time we listen on each channel during a full sweep.
    pub channel_switch_timeout: Duration,
    /// last_channel_switch: The last time we switched a channel during full sweep.
    pub last_channel_switch: DateTime<Utc>,

    /// We go through all channels and listen for new stations/update old
    /// stations every in regular intervals
    pub full_sweep_timeout: Duration,
    /// last_full_sweep: The last time checked all channels for new stations.
    pub last_full_sweep: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> Self {
        let full_sweep_timeout = Duration::hours(1);
        let channel_switch_timeout = Duration::seconds(5);

        // Set the last channel sweep and switch to the past.
        // That way we start with a sweep right away.
        let last_full_sweep = Utc::now();
        let last_channel_switch = Utc::now()
            .checked_sub_signed(Duration::hours(2))
            .expect("This should happen.");

        AppState {
            stations: HashMap::new(),
            devices: HashMap::new(),
            station_device_map: HashMap::new(),
            watched_channels: Vec::new(),
            current_watched_channels: 1,
            full_sweep_timeout,
            channel_switch_timeout,
            last_full_sweep,
            last_channel_switch,
        }
    }

    pub fn should_sweep(&self) -> bool {
        (Utc::now() - self.last_full_sweep) > self.full_sweep_timeout
    }

    pub fn should_switch_channel(&self) -> bool {
        (Utc::now() - self.last_channel_switch) > self.channel_switch_timeout
    }

    pub fn update_watched_channels(&mut self) {
        self.watched_channels = self
            .stations
            .values()
            .filter(|station| station.watch)
            .map(|station| station.channel)
            .collect::<Vec<i32>>();

        self.watched_channels.sort();
        self.watched_channels.dedup();
    }

    pub fn get_next_watched_channel(&mut self) -> i32 {
        if self.watched_channels.is_empty() {
            return 0;
        }

        self.current_watched_channels += 1;
        if self.current_watched_channels as usize >= self.watched_channels.len() {
            self.current_watched_channels = 0;
        }

        self.watched_channels[self.current_watched_channels as usize]
    }
}
