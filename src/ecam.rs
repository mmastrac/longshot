use std::{future::Future, pin::Pin};

use thiserror::Error;

pub enum EcamOutput {
    Ready,
    Packet(Vec<u8>),
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

pub trait Ecam {
    /// Read one item from the ECAM.
    fn read(self: &Self) -> Pin<Box<dyn Future<Output = Result<Option<EcamOutput>, EcamError>> + Send>>;
    /// Send one item to the ECAM.
    fn send(self: &Self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<(), EcamError>> + Send>>;
}
