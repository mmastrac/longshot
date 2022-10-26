use crate::prelude::*;

use thiserror::Error;
use uuid::Uuid;

mod driver;
mod ecam;
mod ecam_bt;
mod ecam_subprocess;
pub mod hardware_enums;
pub mod machine_enum;
mod packet_receiver;

use self::ecam_bt::EcamBT;
pub use driver::{EcamDriver, EcamOutput};
pub use ecam::{Ecam, EcamStatus};
pub use ecam_bt::get_ecam as get_ecam_bt;
pub use ecam_subprocess::connect as get_ecam_subprocess;
pub use packet_receiver::EcamPacketReceiver;

pub async fn ecam_scan() -> Result<(String, Uuid), EcamError> {
    EcamBT::scan().await
}

#[derive(Error, Debug)]
pub enum EcamError {
    #[error("not found")]
    NotFound,
    #[error("timed out")]
    Timeout,
    #[error(transparent)]
    BTError(#[from] btleplug::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}
