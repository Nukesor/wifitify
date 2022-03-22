use anyhow::{bail, Result};
use chrono::Utc;
use clap::Parser;
use crossbeam_channel::{unbounded, RecvTimeoutError};
use libwifi::Frame;
use log::{debug, info, LevelFilter};
use pretty_env_logger::formatted_builder;
use radiotap::Radiotap;

mod cli;
mod config;
mod data;
mod db;
mod device;
mod listener;
mod state;
mod wifi;

use cli::CliArguments;
use db::DbPool;
use device::{supported_channels, switch_channel};
use state::AppState;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Initalize everything
    let (mut state, mut pool) = init_app(opt.verbose).await?;

    // Initialize the channel used to send Wifi frames from the receiver thread.
    // Spawn the packet receiver thread afterwards.
    let (sender, receiver) = unbounded::<(Frame, Radiotap)>();
    listener::init_packet_listener_thread(&opt.device, sender)?;

    // All supported channel of this device and the iterator that's used to walk through those channels.
    let supported_channels = supported_channels(&opt.device)?;
    let mut supported_channel_iter = supported_channels.iter();
    info!("Found supported channels: {:?}", supported_channels);

    // Load all devices and stations from the database.
    // While doing so, we also determine, which channels should be watched depending on the watched
    // stations we get from the database.
    state.init_state(&mut pool, &supported_channels).await?;

    loop {
        let doing_sweep = state.should_sweep();
        // Try to receive for a few milliseconds.
        // Sometimes we might walk over channels that don't have any active devices.
        // If we would keep listening on those devices, we would be wait forever!
        match receiver.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok((frame, radiotap)) => {
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    data::handle_packet(pool_clone, frame, radiotap, doing_sweep).await;
                });
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
                if let Some(channel) = state.get_next_watched_channel() {
                    switch_channel(&opt.device, channel)?;
                    debug!("Switching to channel {}", channel);
                    state.last_channel_switch = Utc::now();
                }
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
            info!("Full sweep finished");
            supported_channel_iter = supported_channels.iter();
            state.update_watched_channels(&supported_channels);
            info!("Watched channels are: {:?}", &state.watched_channels);

            if state.config.collector.always_sweep {
                state.schedule_sweep()
            } else {
                state.last_full_sweep = Utc::now();
            }
            continue;
        };

        switch_channel(&opt.device, next_channel)?;
        debug!("Switching to channel {}", next_channel);
        state.last_channel_switch = Utc::now();
    }
}

/// Init better_panics
/// Initialize logging
/// Create app state
/// Initialize database pool
async fn init_app(verbosity: u8) -> Result<(AppState, DbPool)> {
    // Beautify panics for better debug output.
    better_panic::install();

    // Set the verbosity level and initialize the logger.
    let level = match verbosity {
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

    // Initialize app state and configuration
    let state = AppState::new()?;

    // Initialize the database connection pool and mirror the database state into the state
    let pool: DbPool = db::init_pool(&state.config.database_url).await?;

    Ok((state, pool))
}
