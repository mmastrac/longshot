use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::sync::{mpsc::Receiver, Mutex};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::command::*;

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

#[derive(Debug, PartialEq)]
pub enum EcamStatus {
    Unknown,
    StandBy,
    Ready,
    Busy,
}

impl EcamStatus {
    fn extract(state: &MonitorState) -> EcamStatus {
        if state.state == MachineState::StandBy {
            return EcamStatus::StandBy;
        }
        if state.state == MachineState::Ready {
            return EcamStatus::Ready;
        }
        return EcamStatus::Busy;
    }

    fn matches(&self, state: &MonitorState) -> bool {
        *self == Self::extract(state)
    }
}

/// Async-ish traits for read/write. See https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
/// for some tips on making async trait functions.
pub trait EcamDriver: Send + Sync {
    /// Read one item from the ECAM.
    fn read<'a>(&'a self) -> AsyncFuture<'a, Option<EcamOutput>>;

    /// Write one item to the ECAM.
    fn write<'a>(&'a self, data: Vec<u8>) -> AsyncFuture<'a, ()>;

    /// Scan for the first matching device.
    fn scan<'a>() -> AsyncFuture<'a, (String, Uuid)>
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct Ecam {
    driver: Arc<Box<dyn EcamDriver>>,
    internals: Arc<Mutex<EcamInternals>>,
}

struct EcamInternals {
    last_status: tokio::sync::watch::Receiver<Option<MonitorState>>,
    ready_lock: Arc<tokio::sync::Semaphore>,
}

impl Ecam {
    pub async fn new(driver: Box<dyn EcamDriver>) -> Self {
        let driver = Arc::new(driver);
        let (tx, rx) = tokio::sync::watch::channel(None);
        let ready_lock = Arc::new(tokio::sync::Semaphore::new(1));
        let internals = Arc::new(Mutex::new(EcamInternals {
            last_status: rx,
            ready_lock,
        }));
        let ecam = Ecam { driver, internals };
        let driver = ecam.driver.clone();
        let internals = ecam.internals.clone();
        let mut ready_lock_semaphore = Some(
            internals
                .lock()
                .await
                .ready_lock
                .clone()
                .acquire_owned()
                .await
                .unwrap(),
        );
        tokio::spawn(async move {
            loop {
                if tx.is_closed() {
                    break;
                }
                match driver.read().await? {
                    Some(EcamOutput::Packet(Response::State(x))) => {
                        // println!("{:?}", x);
                        if tx.send(Some(x)).is_err() {
                            break;
                        }
                        ready_lock_semaphore.take();
                    }
                    Some(EcamOutput::Done) => {
                        break;
                    }
                    x => {
                        println!("{:?}", x);
                    }
                }
            }
            println!("Closed");
            Result::<(), EcamError>::Ok(())
        });
        let driver = ecam.driver.clone();
        tokio::spawn(async move {
            loop {
                driver
                    .write(Request::Monitor(MonitorRequestVersion::V2).encode())
                    .await?;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Result::<(), EcamError>::Ok(())
        });
        ecam
    }

    pub async fn wait_for_state(&self, state: EcamStatus) -> Result<(), EcamError> {
        let mut rx = self.internals.lock().await.last_status.clone();
        loop {
            if let Some(test) = rx.borrow().as_ref() {
                if state.matches(test) {
                    return Ok(());
                }
            }
            // TODO: timeout
            rx.changed().await.map_err(|_| EcamError::Unknown)?;
        }
    }

    pub async fn current_state(&self) -> Result<EcamStatus, EcamError> {
        let internals = self.internals.lock().await;
        let rx = internals.last_status.clone();
        drop(
            internals
                .ready_lock
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| EcamError::Unknown)?,
        );
        let ret = if let Some(test) = rx.borrow().as_ref() {
            Ok(EcamStatus::extract(test))
        } else {
            Err(EcamError::Unknown)
        };
        ret
    }

    pub async fn write(&self, request: Request) -> Result<(), EcamError> {
        self.driver.write(request.encode()).await
    }
}

/// Converts a stream into something that can be more easily awaited.
pub struct EcamPacketReceiver {
    rx: Arc<Mutex<Pin<Box<Receiver<EcamOutput>>>>>,
}

impl EcamPacketReceiver {
    pub fn from_stream<T: futures::Stream<Item = EcamOutput> + Unpin + Send + 'static>(
        mut stream: T,
        wrap_start_end: bool,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            if wrap_start_end {
                tx.send(EcamOutput::Ready)
                    .await
                    .expect("Failed to forward notification");
            }
            while let Some(m) = stream.next().await {
                tx.send(m).await.expect("Failed to forward notification");
            }
            if wrap_start_end {
                tx.send(EcamOutput::Done)
                    .await
                    .expect("Failed to forward notification");
            }
        });

        EcamPacketReceiver {
            rx: Arc::new(Mutex::new(Box::pin(rx))),
        }
    }

    pub async fn recv(&self) -> Result<Option<EcamOutput>, EcamError> {
        Ok(self.rx.lock().await.recv().await)
    }
}

#[cfg(test)]
mod test {
    use super::{EcamDriver, EcamError, EcamOutput};
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

    impl EcamDriver for EcamTest {
        fn read<'a>(
            &'a self,
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
            &'a self,
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
        let test = EcamTest::new(vec![EcamOutput::Packet(Response::Raw(vec![]))]);
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
