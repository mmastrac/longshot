//! Brew coffee from the command-line!
//!
//! Longshot is an API and command-line application to brew coffee from the command-line (or whatever
//! front-end is built). At this time it supports DeLonghi ECAM-based Bluetooth-Low-Energy devices, and has only been tested on the
//! Dinamica Plus over Bluetooth.
//!
//! # Examples
//!
//! Monitor a given device:
//! ```text
//! $ longshot monitor --device-name (device)
//! Dispensing... [###############################===========]
//! ```
//! Get the brew information for a given beverage:
//!
//! ```text
//! $ longshot brew  --device-name (device) --beverage regularcoffee
//! ...
//! ```
//!
//! Brew a beverage:
//!
//! ```text
//! $ longshot brew  --device-name (device) --beverage regularcoffee --coffee 180 --taste strong
//! Fetching recipe for RegularCoffee...
//! Fetching recipes...
//! Brewing RegularCoffee with --coffee=180 --taste strong
//! ```

pub mod display;
pub mod ecam;
pub mod logging;
pub mod operations;
mod prelude;
pub mod protocol;
