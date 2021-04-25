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
        5160 => Some(32),
        5170 => Some(34),
        5180 => Some(36),
        5190 => Some(38),
        5200 => Some(40),
        5210 => Some(42),
        5220 => Some(44),
        5230 => Some(46),
        5240 => Some(48),
        5250 => Some(50),
        5260 => Some(52),
        5270 => Some(54),
        5280 => Some(56),
        5290 => Some(58),
        5300 => Some(60),
        5310 => Some(62),
        5320 => Some(64),
        5340 => Some(68),
        5480 => Some(96),
        5500 => Some(100),
        5510 => Some(102),
        5520 => Some(104),
        5530 => Some(106),
        5540 => Some(108),
        5550 => Some(110),
        5560 => Some(112),
        5570 => Some(114),
        5580 => Some(116),
        5590 => Some(118),
        5600 => Some(120),
        5610 => Some(122),
        5620 => Some(124),
        5630 => Some(126),
        5640 => Some(128),
        5660 => Some(132),
        5670 => Some(134),
        5680 => Some(136),
        5690 => Some(138),
        5700 => Some(140),
        5710 => Some(142),
        5720 => Some(144),
        5745 => Some(149),
        5755 => Some(151),
        5765 => Some(153),
        5775 => Some(155),
        5785 => Some(157),
        5795 => Some(159),
        5805 => Some(161),
        5815 => Some(163),
        5825 => Some(165),
        5835 => Some(167),
        5845 => Some(169),
        5855 => Some(171),
        5865 => Some(173),
        5875 => Some(175),
        5885 => Some(177),
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

/// Switch the current channel of a device.
pub fn switch_channel(device: &String, channel: i32) -> Result<()> {
    // Get the list of supported channels via `iwlist $device channel`.
    Command::new("iwlist")
        .arg(device)
        .arg("channel")
        .arg(channel.to_string())
        .output()
        .context("Couldn't switch channels for device. iwconfig command failed.")?;

    Ok(())
}
