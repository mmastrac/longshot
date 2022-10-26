use crate::prelude::*;
use std::{pin::Pin, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::sync::{mpsc::Receiver, Mutex};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::command::*;

mod driver;
mod ecam_bt;
mod ecam_subprocess;
mod packet_receiver;

use self::ecam_bt::EcamBT;
pub use driver::EcamDriver;
pub use ecam_bt::get_ecam as get_ecam_bt;
pub use ecam_subprocess::connect as get_ecam_subprocess;
pub use packet_receiver::EcamPacketReceiver;

pub async fn ecam_scan() -> Result<(String, Uuid), EcamError> {
    EcamBT::scan().await
}

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

#[cfg(test)]
mod test {
    use super::{EcamDriver, EcamError, EcamOutput};
    use crate::command::*;
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
        fn read<'a>(&'a self) -> crate::prelude::AsyncFuture<'a, Option<EcamOutput>> {
            Box::pin(async {
                if self.read_items.lock().unwrap().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(self.read_items.lock().unwrap().remove(0)))
                }
            })
        }

        fn write<'a>(&'a self, data: Vec<u8>) -> crate::prelude::AsyncFuture<'a, ()> {
            self.write_items.lock().unwrap().push(data);
            Box::pin(async { Ok(()) })
        }

        fn scan<'a>() -> crate::prelude::AsyncFuture<'a, (String, uuid::Uuid)>
        where
            Self: Sized,
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
