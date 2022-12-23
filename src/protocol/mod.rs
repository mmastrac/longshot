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
    /// Packet received when pouring CAPPUCCINO milk
    pub const RESPONSE_STATUS_CAPPUCCINO_MILK: [u8; 19] =
        Decoder::Hex.decode(b"d012750f02040100400a040000000000004183");
    /// Packet received after pouring a CAPPUCCINO but before cleaning
    pub const RESPONSE_STATUS_READY_AFTER_CAPPUCCINO: [u8; 19] =
        Decoder::Hex.decode(b"d012750f02040100400700000000000000d621");
    /// Packet received during cleaing
    pub const RESPONSE_STATUS_CLEANING_AFTER_CAPPUCCINO: [u8; 19] =
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
    /// Packet received while shutting down.
    pub const RESPONSE_STATUS_SHUTTING_DOWN_1: [u8; 19] =
        Decoder::Hex.decode(b"d012750f000000000002016400000000007fc5");
    /// Packet received while shutting down.
    pub const RESPONSE_STATUS_SHUTTING_DOWN_2: [u8; 19] =
        Decoder::Hex.decode(b"d012750f0002000000020364000000000019cc");
    /// Packet received while shutting down.
    pub const RESPONSE_STATUS_SHUTTING_DOWN_3: [u8; 19] =
        Decoder::Hex.decode(b"d012750f000000000002066400000000006681");
    /// Packet received during descaling.
    pub const RESPONSE_STATUS_DESCALING_A: [u8; 19] =
        Decoder::Hex.decode(b"d012750f010500040804040000000000001018");
    pub const RESPONSE_STATUS_DESCALING_B: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01450005080408000000000000f076");
    pub const RESPONSE_STATUS_DESCALING_C: [u8; 19] =
        Decoder::Hex.decode(b"d012750f011500040804080000000000007523");
    pub const RESPONSE_STATUS_DESCALING_D: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01030004080409000000000000f12c");
    pub const RESPONSE_STATUS_DESCALING_E: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01010004080409000000000000f7c6");
    pub const RESPONSE_STATUS_DESCALING_F: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01050004080409000000000000fa12");
    pub const RESPONSE_STATUS_DESCALING_G: [u8; 19] =
        Decoder::Hex.decode(b"d012750f014500050804090000000000004817");
    pub const RESPONSE_STATUS_DESCALING_H: [u8; 19] =
        Decoder::Hex.decode(b"d012750f014500050804070000000000007a9f");
    pub const RESPONSE_STATUS_DESCALING_I: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01150004080407000000000000ffca");
    pub const RESPONSE_STATUS_DESCALING_J: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01050004080407000000000000c89a");
    pub const RESPONSE_STATUS_DESCALING_K: [u8; 19] =
        Decoder::Hex.decode(b"d012750f014d000100041100000000000073a3");
    pub const RESPONSE_STATUS_DESCALING_L: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01450001000411000000000000680b");
    pub const RESPONSE_STATUS_DESCALING_M: [u8; 19] =
        Decoder::Hex.decode(b"d012750f01150000000109640000000000588f");
    pub const RESPONSE_STATUS_DESCALING_N: [u8; 19] =
        Decoder::Hex.decode(b"d012750f010500000001096400000000006fdf");
}
