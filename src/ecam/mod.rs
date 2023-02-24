//! Low-level communication with ECAM-based devices.

use std::fmt::Display;

use crate::prelude::*;

use thiserror::Error;

mod driver;
mod ecam_bt;
mod ecam_simulate;
mod ecam_subprocess;
mod ecam_wrapper;
mod packet_receiver;
mod packet_stream;
mod stdin_stream;

pub use self::ecam_bt::EcamBT;
pub use driver::{EcamDriver, EcamDriverOutput};
pub use ecam_simulate::get_ecam_simulator;
pub use ecam_subprocess::connect as get_ecam_subprocess;
pub use ecam_wrapper::{Ecam, EcamOutput, EcamStatus};
pub use packet_receiver::EcamPacketReceiver;
pub use stdin_stream::pipe_stdin;

/// Holds the device name we would like to communicate with.
#[derive(Clone, PartialEq, Eq)]
pub enum EcamId {
    /// 'sim'
    Simulator(String),
    /// 'any'
    Any,
    /// Any non-wildcard string
    Name(String),
}

impl Display for EcamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => f.write_fmt(format_args!("{}", "any")),
            Self::Simulator(sim) => f.write_fmt(format_args!("{}", sim)),
            Self::Name(name) => f.write_fmt(format_args!("{}", name)),
        }
    }
}

impl<S: AsRef<str>> From<S> for EcamId {
    fn from(value: S) -> Self {
        let value = value.as_ref();
        if value.starts_with("sim") {
            Self::Simulator(value.to_string())
        } else if value == "any" {
            Self::Any
        } else {
            Self::Name(value.to_string())
        }
    }
}

pub async fn ecam_scan() -> Result<(String, EcamId), EcamError> {
    EcamBT::scan().await
}

pub async fn ecam_lookup(id: &EcamId, dump_packets: bool) -> Result<Ecam, EcamError> {
    let driver = Box::new(get_ecam_subprocess(id).await?);
    trace_packet!("Got ECAM subprocess");
    Ok(Ecam::new(driver, dump_packets).await)
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
