use crate::prelude::*;
use std::time::Duration;

use async_stream::stream;
use tokio::select;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

use crate::protocol::EcamDriverPacket;

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
                        _ => { warning!("Input error"); }
                    }
                },
                Err(_) => { /* Elapsed */ }
                _ => {
                    break;
                }
            }
        }
        trace_shutdown!("packet_stdio_stream()");
    }
}

/// Pipes an EcamDriver to/from stdio.
pub async fn pipe_stdin<T: EcamDriver>(ecam: T) -> Result<(), Box<dyn std::error::Error>> {
    let mut bt_out = Box::pin(packet_stdio_stream());
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    // Watchdog timer
    std::thread::spawn(move || {
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Err(_) => {
                    trace_shutdown!("pipe_stdin() (watchdog expired)");
                    std::process::exit(1);
                }
                Ok(false) => {
                    break;
                }
                Ok(true) => {}
            }
        }
        trace_shutdown!("pipe_stdin() (watchdog)");
    });

    let keepalive = || drop(tx.send(true));
    loop {
        select! {
            alive = ecam.alive() => {
                keepalive();
                if let Ok(true) = alive {
                    continue;
                } else {
                    break;
                }
            },
            input = ecam.read() => {
                keepalive();
                if let Ok(Some(p)) = input {
                    println!("{}", to_line(p));
                } else {
                    trace_shutdown!("pipe_stdin() (device)");
                    break;
                }
            },
            out = bt_out.next() => {
                keepalive();
                if let Some(value) = out {
                    ecam.write(value).await?;
                } else {
                    trace_shutdown!("pipe_stdin() (input)");
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                keepalive();
            }
        }
    }
    let _ = tx.send(false);
    trace_shutdown!("pipe_stdin()");

    Result::Ok(())
}
