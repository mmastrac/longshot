use std::{future::Future, pin::Pin};

use thiserror::Error;

use crate::command::Response;

#[derive(Debug, PartialEq)]
pub enum EcamOutput {
    Ready,
    Packet(Response),
    Done,
}

#[derive(Error, Debug)]
pub enum EcamError {
    #[error("not found")]
    NotFound,
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
    fn read<'a>(
        self: &'a mut Self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<EcamOutput>, EcamError>> + Send + 'a>>;
    /// Write one item to the ECAM.
    fn write<'a>(
        self: &'a mut Self,
        data: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<(), EcamError>> + Send + 'a>>;
}

async fn ecam_wait_for_status(ecam: &mut dyn Ecam, status: EcamStatus) {}

#[cfg(test)]
mod test {
    use super::{Ecam, EcamError, EcamOutput};
    use crate::command::*;
    use futures::Future;
    use std::pin::Pin;

    #[derive(Default)]
    struct EcamTest {
        pub read_items: Vec<EcamOutput>,
        pub write_items: Vec<Vec<u8>>,
    }

    impl EcamTest {
        pub fn new(items: Vec<EcamOutput>) -> EcamTest {
            let mut read_items = vec![];
            read_items.push(EcamOutput::Ready);
            read_items.extend(items);
            read_items.push(EcamOutput::Done);
            EcamTest {
                read_items,
                write_items: vec![],
            }
        }
    }

    impl Ecam for EcamTest {
        fn read<'a>(
            self: &'a mut Self,
        ) -> Pin<Box<dyn Future<Output = Result<Option<EcamOutput>, EcamError>> + Send + 'a>>
        {
            Box::pin(async {
                if self.read_items.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(self.read_items.remove(0)))
                }
            })
        }

        fn write<'a>(
            self: &'a mut Self,
            data: Vec<u8>,
        ) -> Pin<Box<dyn Future<Output = Result<(), EcamError>> + Send + 'a>> {
            self.write_items.push(data);
            Box::pin(async { Ok(()) })
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
