use anyhow::Result;
use crossbeam_channel::Sender;
use libwifi::Frame;
use radiotap::Radiotap;

use crate::wifi::capture::*;

/// Initialize the thread that's listening for radio packages.
/// All received packets are then send to the main thread via a mpsc channel.
pub fn init_packet_listener_thread(device: &str, sender: Sender<(Frame, Radiotap)>) -> Result<()> {
    // The data capture and parsing logic is running in its own thread.
    // This allows us to have all receiving logic in a non-blocking fashion.
    // The actual handling of the received frames can then be done in an async fashion, since
    // there'll be a lot of I/O wait when interacting with the database.
    let mut capture = get_capture(device)?;

    std::thread::spawn(move || {
        while let Ok(packet) = capture.next() {
            let data = handle_packet(packet);
            if let Ok(data) = data {
                // Send extracted data to the receiver.
                // This only errors if the receiver went away, in which case we just bail.
                if sender.send(data).is_err() {
                    return;
                };
            } else {
                //println!("Got error: {:?}", data);
            }
        }
    });

    Ok(())
}
