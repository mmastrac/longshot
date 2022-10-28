#![allow(dead_code)]

use super::PartialEncode;

#[derive(Clone, Debug, PartialEq)]
pub enum AppControl {
    TurnOn,
    RefreshAppId,
}

impl PartialEncode for AppControl {
    fn partial_encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::TurnOn => out.extend_from_slice(&[2, 1]),
            Self::RefreshAppId => out.extend_from_slice(&[3, 2]),
        }
    }
}
