use anyhow::Result;
use libwifi::frame::components::{MacAddress as LibWifiMacAddress, MacParseError};
use sqlx::{Database, Decode};

use std::str::FromStr;

/// New-type struct so we can implement the database decoder for libwifi's MacAddress struct.
pub struct MacAddress(LibWifiMacAddress);

impl std::ops::Deref for MacAddress {
    type Target = LibWifiMacAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LibWifiMacAddress> for MacAddress {
    fn from(mac: LibWifiMacAddress) -> MacAddress {
        MacAddress(mac)
    }
}

/// Reimplement
impl std::str::FromStr for MacAddress {
    type Err = MacParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(MacAddress(LibWifiMacAddress::from_str(input)?))
    }
}

/// Implement the Database decoder trait for the MacAddress
impl<'r, DB: Database> Decode<'r, DB> for MacAddress
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<MacAddress, Box<dyn std::error::Error + 'static + Send + Sync>> {
        // Get the value as String from the database
        let value = <&str as Decode<DB>>::decode(value)?;

        // Parse that value via FromStr
        Ok(MacAddress::from_str(value)?)
    }
}
