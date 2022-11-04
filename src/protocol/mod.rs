//! Protocols for communication with ECAM-based devices.

mod hardware_enums;
mod machine_enum;
mod packet;
mod request;

pub use hardware_enums::*;
pub use machine_enum::*;
pub use packet::*;
pub use request::*;

#[cfg(test)]
pub mod test {
    use const_decoder::Decoder;

    /// Packet received when a brew response is sent
    pub const RESPONSE_BREW_RECEIVED: [u8; 8] = Decoder::Hex.decode(b"d00783f0010064d9");
    /// Packet received when pouring Cappucino milk
    pub const RESPONSE_STATUS_CAPPUCINO_MILK: [u8; 19] =
        Decoder::Hex.decode(b"d012750f02040100400a040000000000004183");
    /// Packet received after pouring a Cappucino but before cleaning
    pub const RESPONSE_STATUS_READY_AFTER_CAPPUCINO: [u8; 19] =
        Decoder::Hex.decode(b"d012750f02040100400700000000000000d621");
    /// Packet received during cleaing
    pub const RESPONSE_STATUS_CLEANING_AFTER_CAPPUCINO: [u8; 19] =
        Decoder::Hex.decode(b"d012750f04050100400c030900000000001cf0");
    /// Packet received when no alarms are present, and the water spout is removed.
    pub const RESPONSE_STATUS_STANDBY_NO_ALARMS: [u8; 19] =
        Decoder::Hex.decode(b"d012750f000000000000036400000000009080");
    /// Packet received when the water tank is missing, and the water spout is removed.
    pub const RESPONSE_STATUS_STANDBY_NO_WATER_TANK: [u8; 19] =
        Decoder::Hex.decode(b"d012750f00100000000003640000000000a7d0");
    /// Packet received when no alarms are present, and the water spout is present.
    pub const RESPONSE_STATUS_STANDBY_WATER_SPOUT: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01010000000003640000000000d696");
    /// Packet received when the coffee grounds container is missing, and the water spout is present.
    pub const RESPONSE_STATUS_STANDBY_NO_COFFEE_CONTAINER: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01090000000003640000000000cd3e");
}
