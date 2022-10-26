use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use crate::ecam::EcamError;

pub type AsyncFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, EcamError>> + Send + 'a>>;
