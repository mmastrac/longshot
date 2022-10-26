use crate::prelude::*;

use thiserror::Error;
use uuid::Uuid;

mod driver;
mod ecam;
mod ecam_bt;
mod ecam_subprocess;
mod packet_receiver;

use self::ecam_bt::EcamBT;
pub use driver::EcamDriver;
pub use ecam::{Ecam, EcamStatus};
pub use ecam_bt::get_ecam as get_ecam_bt;
pub use ecam_subprocess::connect as get_ecam_subprocess;
pub use packet_receiver::EcamPacketReceiver;

pub async fn ecam_scan() -> Result<(String, Uuid), EcamError> {
    EcamBT::scan().await
}

#[derive(Debug, PartialEq)]
pub enum EcamOutput {
    Ready,
    Packet(crate::command::Response),
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
