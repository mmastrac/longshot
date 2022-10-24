use std::time::Duration;

use async_stream::stream;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

enum InputResult {
    Packet(Vec<u8>),
    Quit,
}

fn parse_line(s: &str) -> Option<InputResult> {
    if s.starts_with("S: ") {
        if let Ok(bytes) = hex::decode(&s[3..]) {
            InputResult::Packet(bytes);
        }
        None
    } else if s.starts_with("Q:") {
        Some(InputResult::Quit)
    } else {
        None
    }
}

pub fn packet_stdio_stream() -> impl Stream<Item = Vec<u8>> {
    println!("R: READY");

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
                        Some(InputResult::Packet(v)) => { yield v; }
                        Some(InputResult::Quit) => { break; }
                        _ => { println!("Input error"); }
                    }
                },
                Err(_) => { /* Elapsed */ }
                _ => {
                    break;
                }
            }
        }
        println!("Exiting stdin loop.");
    }
}
