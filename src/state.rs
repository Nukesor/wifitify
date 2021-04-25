use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

use crate::db::models::{Device, Station};

pub struct AppState {
    /// Our local cache for the station database table.
    pub stations: HashMap<String, Station>,
    /// Our local cache for the device database table.
    pub devices: HashMap<String, Device>,

    /// We go through all channels and listen for new stations/update old
    /// stations every in regular intervals
    pub full_sweep_timeout: Duration,
    /// The time we listen on each channel during a full sweep.
    pub channel_switch_timeout: Duration,
    /// last_full_sweep: The last time checked all channels for new stations.
    pub last_full_sweep: DateTime<Utc>,
    /// last_channel_switch: The last time we switched a channel during full sweep.
    pub last_channel_switch: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> Self {
        let full_sweep_timeout = Duration::hours(1);
        let channel_switch_timeout = Duration::seconds(10);

        // Set the last channel sweep and switch to the past.
        // That way we start with a sweep right away.
        let last_full_sweep = Utc::now()
            .checked_sub_signed(full_sweep_timeout)
            .expect("This should happen.");
        let last_channel_switch = Utc::now()
            .checked_sub_signed(Duration::hours(2))
            .expect("This should happen.");

        AppState {
            stations: HashMap::new(),
            devices: HashMap::new(),
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
}
