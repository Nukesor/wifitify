use std::collections::HashMap;

pub mod capture;

/// Map a MHz number to the respective WiFi channel.
pub fn get_mhz_to_channel(mhz: u16) -> Option<i32> {
    let mut map = HashMap::new();
    map.insert(2412, 1);
    map.insert(2417, 2);
    map.insert(2422, 3);
    map.insert(2427, 4);
    map.insert(2432, 5);
    map.insert(2437, 6);
    map.insert(2442, 7);
    map.insert(2447, 8);
    map.insert(2452, 9);
    map.insert(2457, 10);
    map.insert(2462, 11);
    map.insert(2467, 12);
    map.insert(2472, 13);
    map.insert(2484, 14);

    map.get(&mhz).copied()
}
