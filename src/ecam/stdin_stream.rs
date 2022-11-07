use crate::prelude::*;
use async_stream::stream;
use std::time::Duration;
use tokio::join;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tuples::*;

use crate::protocol::EcamDriverPacket;

use super::{EcamDriver, EcamDriverOutput, EcamError};

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

macro_rules! spawn_loop {
    ($name:literal, $tx:expr, $async:block) => {{
        let tx = $tx.clone();
        async move {
            while let Ok(_) = tx.send(true) {
                $async
            }
            trace_shutdown!($name);
            let _ = tx.send(false);
            Result::<(), EcamError>::Ok(())
        }
    }};
}

/// Pipes an EcamDriver to/from stdio.
pub async fn pipe_stdin<T: EcamDriver + 'static>(
    ecam: T,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bt_out = Box::pin(packet_stdio_stream());
    let ecam = Arc::new(Box::new(ecam));
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    // Watchdog timer: if we don't get _some_ event within the timeout, we assume that things havegone sideways
    // in the underlying driver.
    std::thread::spawn(move || {
        loop {
            match rx.recv_timeout(Duration::from_millis(500)) {
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

    let ecam2 = ecam.clone();
    let a = spawn_loop!("alive", tx, {
        if !(ecam2.alive().await?) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    });
    let ecam2 = ecam.clone();
    let b = spawn_loop!("device read", tx, {
        if let Some(p) = ecam2.read().await? {
            println!("{}", to_line(p));
        } else {
            break;
        }
    });
    let c = spawn_loop!("stdio read", tx, {
        if let Some(value) = bt_out.next().await {
            ecam.write(value).await?;
        } else {
            break;
        }
    });

    let x: Result<_, EcamError> = join!(a, b, c).map(|x| x).transpose();
    x?;

    trace_shutdown!("pipe_stdin()");

    Result::Ok(())
}
