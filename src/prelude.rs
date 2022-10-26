use crate::ecam::EcamError;

pub use std::future::Future;
pub use std::{pin::Pin, sync::Arc};
pub use tokio_stream::{Stream, StreamExt};

pub type AsyncFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, EcamError>> + Send + 'a>>;
