//! Universal imports for this crate.

use crate::ecam::EcamError;

pub use std::future::Future;
pub use std::{pin::Pin, sync::Arc, time::Duration};
pub use tokio_stream::{Stream, StreamExt};

pub use crate::util::CollectMapJoin;
pub use crate::{info, trace_packet, trace_shutdown, warning};

pub type AsyncFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, EcamError>> + Send + 'a>>;
