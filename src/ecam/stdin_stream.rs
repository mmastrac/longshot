use std::time::Duration;

use async_stream::stream;
use tokio::select;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

use crate::{protocol::EcamDriverPacket, trace_packet};

use super::{EcamDriver, EcamDriverOutput};

/// Converts a stdio line to an EcamDriverOutput.
fn parse_line(s: &str) -> Option<EcamDriverOutput> {
    if s == "R: READY" {
        Some(EcamDriverOutput::Ready)
    } else if let Some(s) = s.strip_prefix("S: ") {
        if let Ok(bytes) = hex::decode(&s) {
            Some(EcamDriverOutput::Packet(EcamDriverPacket::from_vec(bytes)))
        } else {
            None
        }
    } else if s.starts_with("Q:") {
        Some(EcamDriverOutput::Done)
    } else {
        None
    }
}

/// Converts an EcamDriverOutput to a stdio line.
fn to_line(output: EcamDriverOutput) -> String {
    match output {
        EcamDriverOutput::Ready => "R: READY".to_owned(),
        EcamDriverOutput::Done => "Q:".to_owned(),
        EcamDriverOutput::Packet(p) => format!("R: {}", p.stringify()),
    }
}

fn packet_stdio_stream() -> impl Stream<Item = EcamDriverPacket> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || {
        for l in std::io::stdin().lines() {
            if tx.blocking_send(l).is_err() {
                break;
            }
        }
    });

    let mut lines = ReceiverStream::new(rx);
    stream! {
        loop {
            match tokio::time::timeout(Duration::from_millis(250), lines.next()).await {
                Ok(Some(Ok(s))) => {
                    match parse_line(&s) {
                        Some(EcamDriverOutput::Packet(v)) => { yield v; }
                        Some(EcamDriverOutput::Done) => { break; }
                        _ => { println!("Input error"); }
                    }
                },
                Err(_) => { /* Elapsed */ }
                _ => {
                    break;
                }
            }
        }
        trace_packet!("Exiting stdin loop.");
    }
}

/// Pipes an EcamDriver to/from stdio.
pub async fn pipe_stdin<T: EcamDriver>(ecam: T) -> Result<(), Box<dyn std::error::Error>> {
    let mut bt_out = Box::pin(packet_stdio_stream());

    loop {
        select! {
            input = ecam.read() => {
                if let Ok(Some(p)) = input {
                    println!("{}", to_line(p));
                } else {
                    println!("Device closed");
                    break;
                }
            },
            out = bt_out.next() => {
                if let Some(value) = out {
                    ecam.write(value).await?;
                } else {
                    println!("Input closed");
                    break;
                }
            }
        }
    }
    println!("Pipe shutting down");

    Result::Ok(())
}
