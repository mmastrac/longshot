use crate::prelude::*;

use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum EcamOutput {
    Ready,
    Packet(crate::command::Response),
    Logging(String),
    Done,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::command::*;
    use crate::ecam::EcamError;
    use std::sync::Mutex;

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
