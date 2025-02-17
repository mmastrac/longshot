use crate::prelude::*;
use keepcalm::SharedMut;
use tokio::sync::mpsc::Receiver;

use crate::ecam::{EcamDriverOutput, EcamError};

/// Converts a stream into something that can be more easily awaited. In addition, it can optionally add
/// [`EcamDriverOutput::Ready`] and [`EcamDriverOutput::Done`] packets to the start and end of the stream.
pub struct EcamPacketReceiver {
    rx: SharedMut<Receiver<EcamDriverOutput>>,
}

impl EcamPacketReceiver {
    pub fn from_stream<T: futures::Stream<Item = EcamDriverOutput> + Unpin + Send + 'static>(
        mut stream: T,
        wrap_start_end: bool,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            if wrap_start_end {
                tx.send(EcamDriverOutput::Ready)
                    .await
                    .expect("Failed to forward notification");
            }
            while let Some(m) = stream.next().await {
                tx.send(m).await.expect("Failed to forward notification");
            }
            trace_shutdown!("EcamPacketReceiver");
            if wrap_start_end {
                tx.send(EcamDriverOutput::Done)
                    .await
                    .expect("Failed to forward notification");
            }
        });

        EcamPacketReceiver {
            rx: SharedMut::new(rx),
        }
    }

    pub async fn recv(&self) -> Result<Option<EcamDriverOutput>, EcamError> {
        Ok(self.rx.write().recv().await)
    }
}
