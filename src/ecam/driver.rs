use crate::prelude::*;

use uuid::Uuid;
use crate::ecam::EcamOutput;

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
