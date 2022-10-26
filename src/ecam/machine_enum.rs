use std::fmt::Debug;

/// Wraps a machine enumeration that may have unknown values.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum MachineEnum<T> {
    Value(T),
    Unknown(u8),
}

impl<T: TryFrom<u8>> MachineEnum<T> {
    pub fn decode(value: u8) -> Self {
        if let Ok(value) = T::try_from(value) {
            MachineEnum::Value(value)
        } else {
            MachineEnum::Unknown(value)
        }
    }
}

impl<T: Debug> Debug for MachineEnum<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(t) => t.fmt(f),
            Self::Unknown(v) => format!("Unknown({})", v).fmt(f),
        }
    }
}

impl<T: PartialEq> PartialEq<T> for MachineEnum<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Value(t) => t.eq(other),
            Self::Unknown(_v) => false,
        }
    }
}
