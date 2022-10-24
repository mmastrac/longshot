use std::future::Future;

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

trait Ecam {
    /// Read one item from the ECAM.
    fn read(self: &Self) -> Box<dyn Future<Output = Result<EcamOutput, EcamError>> + Send>;
    /// Send one item to the ECAM.
    fn send(self: &Self, data: Vec<u8>) -> Box<dyn Future<Output = Result<(), EcamError>> + Send>;
}
