/// This is our representation of a MAC-address
#[derive(Clone, Debug)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Get the mac address from a 6 byte slice.
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut address: [u8; 6] = [0; 6];
        address.clone_from_slice(&slice[0..6]);

        MacAddress(address)
    }

    /// Return the MacAddress' bytes in easily readable Hex-code
    pub fn to_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    /// Check whether this MAC addresses the whole network.
    pub fn is_broadcast(&self) -> bool {
        self.0 == [255, 255, 255, 255, 255, 255]
    }
}
