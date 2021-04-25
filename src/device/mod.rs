use anyhow::{Context, Result};

use std::process::Command;

/// Map a MHz number to the respective WiFi channel.
pub fn get_mhz_to_channel(mhz: u16) -> Option<i32> {
    match mhz {
        2412 => Some(1),
        2417 => Some(2),
        2422 => Some(3),
        2427 => Some(4),
        2432 => Some(5),
        2437 => Some(6),
        2442 => Some(7),
        2447 => Some(8),
        2452 => Some(9),
        2457 => Some(10),
        2462 => Some(11),
        2467 => Some(12),
        2472 => Some(13),
        2484 => Some(14),
        _ => None,
    }
}

/// Map a MHz number to the respective WiFi channel.
pub fn supported_channels(device: &String) -> Result<Vec<i32>> {
    // Get the list of supported channels via `iwlist $device channel`.
    let output = Command::new("iwlist")
        .arg(device)
        .arg("channel")
        .output()
        .context("Couldn't detect supported channels for device. iwlist command failed.")?;

    let output = String::from_utf8(output.stdout)
        .context("Got invalid utf8 from 'iwlist $device channel' command.")?;

    let mut channels = Vec::new();

    // Each channel is listed on its own line
    // Such a line looks like this:
    // `          Channel 01 : 2.412 GHz`
    let lines = output.split('\n');

    for mut line in lines {
        // Remove any trailing spaces. That way we can do a starts_with check.
        line = line.trim();

        // Only look at actual channel lines.
        if !line.starts_with("Channel") {
            continue;
        }
        // Split the line by space. The channel is the second word in the line.
        let mut splitted = line.split(' ');

        // Get the channel string and remove any trailing zeros.
        let channel = splitted.nth(1).context(format!(
            "Got incorrectly formatted channel line from iwlist:\n{}",
            &line
        ))?;
        let channel: i32 = channel
            .trim_matches('0')
            .parse()
            .context(format!("Got invalid channel number from line: {}", &line))?;

        channels.push(channel);
    }

    Ok(channels)
}
