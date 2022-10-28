use std::fmt::Debug;

use crate::protocol::request::{PartialDecode, PartialEncode};

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
    #[cfg(test)]
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
    pub bytes: Vec<u8>,
}

impl<T> EcamPacket<T> {
    pub fn from_undecodeable_bytes(input: &[u8]) -> EcamPacket<T> {
        let bytes = input.to_vec();
        EcamPacket {
            representation: None,
            bytes,
        }
    }
}

impl<T: PartialDecode<T>> EcamPacket<T> {
    pub fn from_bytes(mut input: &[u8]) -> EcamPacket<T> {
        let bytes = input.to_vec();
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
        let bytes = representation.encode();
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
        EcamDriverPacket::from_vec(packet.bytes)
    }
}

pub fn checksum(buffer: &[u8]) -> [u8; 2] {
    let mut i: u16 = 7439;
    for x in buffer {
        let i3 = ((i << 8) | (i >> 8)) ^ (*x as u16);
        let i4 = i3 ^ ((i3 & 255) >> 4);
        let i5 = i4 ^ (i4 << 12);
        i = i5 ^ ((i5 & 255) << 5);
    }

    [(i >> 8) as u8, (i & 0xff) as u8]
}

fn packetize(buffer: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = vec![
        0x0d,
        (buffer.len() + 3).try_into().expect("Packet too large"),
    ];
    out.extend_from_slice(buffer);
    out.extend_from_slice(&checksum(&out));
    out
}

fn stringify(buffer: &[u8]) -> String {
    buffer
        .iter()
        .map(|n| format!("{:02x}", n))
        .collect::<String>()
}

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
        .map(|(i, b)| {
            if *b >= 20 && *b < 127 {
                *b as char
            } else {
                '.'
            }
        })
        .collect::<String>();
    return format!("|{}| |{}|", s1, s2).to_owned();
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
    }
}
