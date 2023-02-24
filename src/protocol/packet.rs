use crate::protocol::request::{PartialDecode, PartialEncode};
use crc::Crc;
use std::fmt::Debug;

#[derive(Clone, Eq, PartialEq)]
/// A simple byte-based driver packet, with header, length and checksum.
pub struct EcamDriverPacket {
    pub(crate) bytes: Vec<u8>,
}

impl Debug for EcamDriverPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hexdump(&self.bytes))
    }
}

impl EcamDriverPacket {
    pub fn from_slice(bytes: &[u8]) -> Self {
        EcamDriverPacket {
            bytes: bytes.into(),
        }
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        EcamDriverPacket { bytes }
    }

    pub fn stringify(&self) -> String {
        stringify(&self.bytes)
    }

    pub fn packetize(&self) -> Vec<u8> {
        packetize(&self.bytes)
    }
}

/// A packet that may have a representation attached, allowing us to parse a packet once and only once.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EcamPacket<T> {
    pub representation: Option<T>,
    pub bytes: EcamDriverPacket,
}

impl<T> EcamPacket<T> {
    #[cfg(test)]
    pub fn from_raw(input: &[u8]) -> EcamPacket<T> {
        let bytes = EcamDriverPacket::from_vec(input.to_vec());
        EcamPacket {
            representation: None,
            bytes,
        }
    }
}

impl<T: PartialDecode<T>> EcamPacket<T> {
    pub fn from_bytes(mut input: &[u8]) -> EcamPacket<T> {
        let bytes = EcamDriverPacket::from_vec(input.to_vec());
        let input = &mut input;
        let representation = <T>::partial_decode(input);
        EcamPacket {
            representation,
            bytes,
        }
    }
}

impl<T: PartialEncode> EcamPacket<T> {
    pub fn from_represenation(representation: T) -> EcamPacket<T> {
        let bytes = EcamDriverPacket::from_vec(representation.encode());
        EcamPacket {
            representation: Some(representation),
            bytes,
        }
    }
}

impl<T: PartialDecode<T>> From<EcamDriverPacket> for EcamPacket<T> {
    fn from(packet: EcamDriverPacket) -> Self {
        EcamPacket::from_bytes(&packet.bytes)
    }
}

impl<T> From<EcamPacket<T>> for EcamDriverPacket {
    fn from(packet: EcamPacket<T>) -> Self {
        packet.bytes
    }
}

pub const CRC_ALGO: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_SPI_FUJITSU);

/// Computes the checksum from a partial packet. Note that the checksum used here is
/// equivalent to the `CRC_16_SPI_FUJITSU` definition (initial 0x1d0f, poly 0x1021).
pub fn checksum(buffer: &[u8]) -> [u8; 2] {
    let i = CRC_ALGO.checksum(buffer);
    [(i >> 8) as u8, (i & 0xff) as u8]
}

/// Returns the contents of the packet, minus header and checksum.
pub fn unwrap_packet<T: ?Sized>(buffer: &T) -> &[u8]
where
    T: AsRef<[u8]>,
{
    let u: &[u8] = buffer.as_ref();
    &u[2..u.len() - 2]
}

fn packetize(buffer: &[u8]) -> Vec<u8> {
    let mut out = [
        &[
            0x0d,
            (buffer.len() + 3).try_into().expect("Packet too large"),
        ],
        buffer,
    ]
    .concat();
    out.extend_from_slice(&checksum(&out));
    out
}

fn stringify(buffer: &[u8]) -> String {
    buffer
        .iter()
        .map(|n| format!("{:02x}", n))
        .collect::<String>()
}

/// Dumps a packet to a readable hex form.
pub fn hexdump(buffer: &[u8]) -> String {
    let maybe_space = |i| if i > 0 && i % 8 == 0 { " " } else { "" };
    let s1: String = buffer
        .iter()
        .enumerate()
        .map(|(i, b)| format!("{}{:02x}", maybe_space(i), b))
        .collect::<String>();
    let s2: String = buffer
        .iter()
        .enumerate()
        .map(|(_i, b)| {
            if *b >= 32 && *b < 127 {
                *b as char
            } else {
                '.'
            }
        })
        .collect::<String>();
    format!("|{}| |{}|", s1, s2)
}

#[cfg(test)]
pub mod test {
    use super::{checksum, packetize};

    pub fn from_hex_str(s: &str) -> Vec<u8> {
        hex::decode(s.replace(' ', "")).unwrap()
    }

    #[test]
    pub fn test_checksum() {
        assert_eq!(
            checksum(&from_hex_str("0d 0f 83 f0 02 01 01 00 67 02 02 00 00 06")),
            [0x77, 0xff]
        );
        assert_eq!(
            checksum(&from_hex_str("0d 0d 83 f0 05 01 01 00 78 00 00 06")),
            [0xc4, 0x7e]
        );
        assert_eq!(checksum(&from_hex_str("0d 07 84 0f 02 01")), [0x55, 0x12]);
    }

    #[test]
    pub fn test_packetize() {
        assert_eq!(
            packetize(&from_hex_str("83 f0 02 01 01 00 67 02 02 00 00 06")),
            from_hex_str("0d 0f 83 f0 02 01 01 00 67 02 02 00 00 06 77 ff")
        );
        assert_eq!(
            packetize(&from_hex_str("83 f0 05 01 01 00 78 00 00 06")),
            from_hex_str("0d 0d 83 f0 05 01 01 00 78 00 00 06 c4 7e")
        );
        assert_eq!(
            packetize(&from_hex_str("84 0f 02 01")),
            from_hex_str("0d 07 84 0f 02 01 55 12")
        );
        assert_eq!(
            packetize(&from_hex_str("75 f0")),
            from_hex_str("0d 05 75 f0 c4 d5")
        );
    }
}
