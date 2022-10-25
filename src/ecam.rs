use std::{future::Future, pin::Pin, sync::Arc};

use thiserror::Error;
use tokio::sync::{mpsc::Receiver, Mutex};
use uuid::Uuid;

use crate::command::Response;

pub type AsyncFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, EcamError>> + Send + 'a>>;

#[derive(Debug, PartialEq)]
pub enum EcamOutput {
    Ready,
    Packet(Response),
    Logging(String),
    Done,
}

#[derive(Error, Debug)]
pub enum EcamError {
    #[error("not found")]
    NotFound,
    #[error("timed out")]
    Timeout,
    #[error(transparent)]
    BTError(#[from] btleplug::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}

pub enum EcamStatus {
    Off,
    Ready,
}

/// Async-ish traits for read/write. See https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
/// for some tips on making async trait functions.
pub trait Ecam {
    /// Read one item from the ECAM.
    fn read<'a>(self: &'a Self) -> AsyncFuture<'a, Option<EcamOutput>>;

    /// Write one item to the ECAM.
    fn write<'a>(self: &'a Self, data: Vec<u8>) -> AsyncFuture<'a, ()>;

    /// Scan for the first matching device.
    fn scan<'a>() -> AsyncFuture<'a, (String, Uuid)>
    where
        Self: Sized;
}

async fn ecam_wait_for_status(ecam: &mut dyn Ecam, status: EcamStatus) {}

pub struct EcamPacketReceiver {
    rx: Arc<Mutex<Pin<Box<Receiver<EcamOutput>>>>>,
}

/// Converts a stream into something that can be more easily awaited.
impl EcamPacketReceiver {
    pub fn from_stream<T: tokio_stream::StreamExt<Item = EcamOutput> + Unpin + Send + 'static>(
        mut stream: T,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            while let Some(m) = stream.next().await {
                tx.send(m).await.expect("Failed to forward notification");
            }
        });

        EcamPacketReceiver {
            rx: Arc::new(Mutex::new(Box::pin(rx))),
        }
    }

    pub async fn recv(self: &Self) -> Result<Option<EcamOutput>, EcamError> {
        Ok(self.rx.lock().await.recv().await)
    }
}

#[cfg(test)]
mod test {
    use super::{Ecam, EcamError, EcamOutput};
    use crate::command::*;
    use futures::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct EcamTest {
        pub read_items: Arc<Mutex<Vec<EcamOutput>>>,
        pub write_items: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl EcamTest {
        pub fn new(items: Vec<EcamOutput>) -> EcamTest {
            let mut read_items = vec![];
            read_items.push(EcamOutput::Ready);
            read_items.extend(items);
            read_items.push(EcamOutput::Done);
            EcamTest {
                read_items: Arc::new(Mutex::new(read_items)),
                write_items: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    impl Ecam for EcamTest {
        fn read<'a>(
            self: &'a Self,
        ) -> Pin<Box<dyn Future<Output = Result<Option<EcamOutput>, EcamError>> + Send + 'a>>
        {
            Box::pin(async {
                if self.read_items.lock().unwrap().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(self.read_items.lock().unwrap().remove(0)))
                }
            })
        }

        fn write<'a>(
            self: &'a Self,
            data: Vec<u8>,
        ) -> Pin<Box<dyn Future<Output = Result<(), EcamError>> + Send + 'a>> {
            self.write_items.lock().unwrap().push(data);
            Box::pin(async { Ok(()) })
        }

        fn scan<'a>(
        ) -> Pin<Box<dyn Future<Output = Result<(String, uuid::Uuid), EcamError>> + Send + 'a>>
        {
            Box::pin(async { Err(EcamError::NotFound) })
        }
    }

    #[tokio::test]
    async fn test_read() -> Result<(), EcamError> {
        let mut test = EcamTest::new(vec![EcamOutput::Packet(Response::Raw(vec![]))]);
        assert_eq!(
            EcamOutput::Ready,
            test.read().await?.expect("expected item")
        );
        assert_eq!(
            EcamOutput::Packet(Response::Raw(vec![])),
            test.read().await?.expect("expected item")
        );
        assert_eq!(EcamOutput::Done, test.read().await?.expect("expected item"));
        assert_eq!(None, test.read().await?);
        Ok(())
    }
}
