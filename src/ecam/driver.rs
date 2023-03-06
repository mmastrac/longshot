use crate::{prelude::*, protocol::*};

use super::EcamId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EcamDriverOutput {
    Ready,
    Packet(EcamDriverPacket),
    Done,
}

/// Async-ish traits for read/write. See <https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/>
/// for some tips on making async trait functions.
pub trait EcamDriver: Send + Sync {
    /// Read one item from the ECAM.
    fn read(&self) -> AsyncFuture<Option<EcamDriverOutput>>;

    /// Write one item to the ECAM.
    fn write(&self, data: EcamDriverPacket) -> AsyncFuture<()>;

    /// Returns true if the driver is alive.
    fn alive(&self) -> AsyncFuture<bool>;

    /// Scan for the first matching device.
    fn scan<'a>() -> AsyncFuture<'a, (String, EcamId)>
    where
        Self: Sized;
}

#[cfg(test)]
mod test {
    use keepcalm::SharedMut;
    use super::*;
    use crate::ecam::EcamError;

    struct EcamTest {
        pub read_items: SharedMut<Vec<EcamDriverOutput>>,
        pub write_items: SharedMut<Vec<EcamDriverPacket>>,
    }

    impl EcamTest {
        pub fn new(items: Vec<EcamDriverOutput>) -> EcamTest {
            let mut read_items = vec![];
            read_items.push(EcamDriverOutput::Ready);
            read_items.extend(items);
            read_items.push(EcamDriverOutput::Done);
            EcamTest {
                read_items: SharedMut::new(read_items),
                write_items: SharedMut::new(vec![]),
            }
        }
    }

    impl EcamDriver for EcamTest {
        fn read(&self) -> crate::prelude::AsyncFuture<Option<EcamDriverOutput>> {
            Box::pin(async {
                if self.read_items.read().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(self.read_items.write().remove(0)))
                }
            })
        }

        fn write(&self, data: EcamDriverPacket) -> crate::prelude::AsyncFuture<()> {
            self.write_items.write().push(data);
            Box::pin(async { Ok(()) })
        }

        fn alive(&self) -> AsyncFuture<bool> {
            Box::pin(async { Ok(true) })
        }

        fn scan<'a>() -> crate::prelude::AsyncFuture<'a, (String, EcamId)>
        where
            Self: Sized,
        {
            Box::pin(async { Err(EcamError::NotFound) })
        }
    }

    #[tokio::test]
    async fn test_read() -> Result<(), EcamError> {
        let test = EcamTest::new(vec![EcamDriverOutput::Packet(
            EcamDriverPacket::from_slice(&[]),
        )]);
        assert_eq!(
            EcamDriverOutput::Ready,
            test.read().await?.expect("expected item")
        );
        assert_eq!(
            EcamDriverOutput::Packet(EcamDriverPacket::from_slice(&[])),
            test.read().await?.expect("expected item")
        );
        assert_eq!(
            EcamDriverOutput::Done,
            test.read().await?.expect("expected item")
        );
        assert_eq!(None, test.read().await?);
        Ok(())
    }
}
