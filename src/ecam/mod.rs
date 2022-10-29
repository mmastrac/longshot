use crate::prelude::*;

use thiserror::Error;
use uuid::Uuid;

mod driver;
mod ecam;
mod ecam_bt;
mod ecam_simulate;
mod ecam_subprocess;
mod packet_receiver;
mod stdin_stream;

use self::ecam_bt::EcamBT;
pub use driver::{EcamDriver, EcamDriverOutput};
pub use ecam::{Ecam, EcamOutput, EcamStatus};
pub use ecam_bt::get_ecam as get_ecam_bt;
pub use ecam_simulate::get_ecam_simulator;
pub use ecam_subprocess::connect as get_ecam_subprocess;
pub use packet_receiver::EcamPacketReceiver;
pub use stdin_stream::pipe_stdin;

pub async fn ecam_scan() -> Result<(String, Uuid), EcamError> {
    EcamBT::scan().await
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
