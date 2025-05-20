#![allow(dead_code)]

use super::PartialEncode;

/// Operations used by the application for various purposes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppControl {
    /// Turns the machine on.
    TurnOn,
    /// Uncertain, but sent by the application.
    RefreshAppId,
    /// Custom command.
    Custom(u8, u8),
}

impl PartialEncode for AppControl {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::TurnOn => out.extend_from_slice(&[2, 1]),
            Self::RefreshAppId => out.extend_from_slice(&[3, 2]),
            Self::Custom(a, b) => out.extend_from_slice(&[*a, *b]),
        }
    }
}
