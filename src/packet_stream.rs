use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

pub fn packet_stdio_stream() -> impl Stream<Item = Vec<u8>> {
    println!("R: READY");

    let (tx, mut rx) = mpsc::channel(2);
    let h1 = tokio::task::spawn_blocking(move || loop {
        let lines = std::io::stdin().lines();
        for s in lines {
            if let Ok(s) = s {
                tx.blocking_send(s).expect("Failed to send");
            } else {
                return ();
            }
        }
    });

    // Listen to the stdin lines
    let (tx2, rx2) = mpsc::channel(2);
    let h2 = tokio::spawn(async move {
        while let Some(s) = rx.recv().await {
            if s.starts_with("S: ") {
                if let Ok(bytes) = hex::decode(&s[3..]) {
                    tx2.send(bytes).await;
                } else {
                    println!("Invalid hex");
                }
            } else if s.starts_with("Q: ") {
                return ();
            } else {
                println!("Invalid input");
            }
        }
    });

    ReceiverStream::new(rx2)
}
