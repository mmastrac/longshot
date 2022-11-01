//! Longshot is an API and command-line application to brew coffee from the command-line. At this
//! time it supports DeLonghi ECAM-based devices, and has only been tested on the Dinamica Plus.

pub mod display;
pub mod ecam;
pub mod logging;
pub mod operations;
pub mod prelude;
pub mod protocol;
