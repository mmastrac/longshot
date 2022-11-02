use crate::prelude::*;

use async_stream::stream;
use futures::{Stream, StreamExt};

use crate::protocol::{hexdump, EcamDriverPacket};

const SYNC_BYTE: u8 = 0xd0;

/// Builds a packet from collections of bytes.
///
/// The [`PacketBuilder`] assumes that packets are aligned to the input chunks, and that a starting chunk
/// that doesn't start with the sync byte, is corrupted or orphaned.
///
/// A starting chunk is defined as the next chunk recieved after a packet is emitted.
#[derive(Default)]
struct PacketBuilder {
    packet_buffer: Vec<u8>,
    offset: usize,
}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder::default()
    }

    pub fn is_empty(&self) -> bool {
        self.packet_buffer.is_empty()
    }

    /// Accumulates a single packet chunk, returning the entire packet as a [`Vec<u8>`] if it is complete.
    pub fn accumulate(&mut self, chunk: &[u8]) -> Option<Vec<u8>> {
        self.packet_buffer.extend_from_slice(chunk);
        let mut p = self.current_packet();

        // If we're not starting on a sync byte, eat bytes until we are.
        while p.len() > 1 && p[0] != SYNC_BYTE {
            self.offset += 1;
            p = self.current_packet();
        }

        if p.len() > 2 {
            let packet_size = p[1] as usize;
            if packet_size < p.len() {
                // We have a full packet, so take what we need
                let offset = std::mem::take(&mut self.offset);
                let packet_buffer = std::mem::take(&mut self.packet_buffer);
                // Optimization: we have exactly the packet we wanted, so just return the buffer
                if offset == 0 && packet_buffer.len() == packet_size + 1 {
                    return Some(packet_buffer);
                }
                return Some(packet_buffer[offset..=offset + packet_size].to_vec());
            }
        }

        None
    }

    fn current_packet(&self) -> &[u8] {
        &self.packet_buffer[self.offset..]
    }
}

/// Converts a stream of raw bytes into a stream of decoded packets.
pub fn packet_stream<T>(mut n: T) -> impl Stream<Item = EcamDriverPacket>
    where T: Stream<Item = Vec<u8>> + StreamExt + std::marker::Unpin {

    stream! {
        let mut p = PacketBuilder::new();
        while let Some(m) = n.next().await {
            if let Some(v) = p.accumulate(&m) {
                yield EcamDriverPacket::from_vec(v);
            }
        }
        trace_packet!("Main receive loop shutting down");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(vec![SYNC_BYTE, 2, 2])]
    #[case(vec![SYNC_BYTE, 3, 10, 20])]
    #[case(vec![SYNC_BYTE, 4, 10, 20, 30])]
    #[case(vec![SYNC_BYTE, 5, 10, 20, 30, 40])]
    #[case(vec![SYNC_BYTE, 7, 0x84, 0x0f, 0x02, 0x01, 0x55, 0x12])]
    fn packet_accumulate_exact(#[case] bytes: Vec<u8>) {
        let mut p = PacketBuilder::new();
        assert_eq!(Some(bytes.clone()), p.accumulate(&bytes));
        assert!(p.is_empty());
    }

    /// Test that extra bytes are tossed away
    #[rstest]
    #[case(vec![SYNC_BYTE, 2, 2, 99, 99, 99])]
    #[case(vec![SYNC_BYTE, 3, 10, 20, 99, 99])]
    #[case(vec![SYNC_BYTE, 4, 10, 20, 30, 99])]
    #[case(vec![SYNC_BYTE, 5, 10, 20, 30, 40, 99, 99, 99])]
    fn packet_accumulate_too_many(#[case] bytes: Vec<u8>) {
        let mut p = PacketBuilder::new();
        let len = bytes[1] as usize;
        let out = bytes[0..len + 1].to_vec();
        assert_eq!(Some(out), p.accumulate(&bytes));
        assert!(p.is_empty());
    }

    /// Ensure that we parse this packet correctly regardless of how it is chunked, and with or without garbage before/after.
    #[rstest]
    fn chunked_packet(
        #[values(true, false)] garbage_before: bool,
        #[values(true, false)] garbage_after: bool,
    ) {
        let mut packet = vec![SYNC_BYTE, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = packet.clone();
        if garbage_before {
            let mut tmp = vec![1, 2, 3];
            tmp.splice(0..0, packet);
            packet = tmp;
        }
        if garbage_after {
            packet.extend_from_slice(&[1, 2, 3]);
        }
        println!("{:?}", packet);
        for i in 0..7 {
            let mut p = PacketBuilder::new();
            assert!(p.accumulate(&packet[..i]).is_none());
            assert_eq!(Some(expected.clone()), p.accumulate(&packet[i..]));
            assert!(p.is_empty());
        }
    }
}
