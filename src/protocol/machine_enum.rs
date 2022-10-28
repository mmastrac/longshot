use std::fmt::Debug;

/// Wraps a machine enumeration that may have unknown values.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    Value(T),
    Unknown(u8),
}

impl<T> MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    pub fn decode(value: u8) -> Self {
        if let Ok(value) = T::try_from(value) {
            MachineEnum::Value(value)
        } else {
            MachineEnum::Unknown(value)
        }
    }
}

impl<T> From<T> for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn from(t: T) -> Self {
        MachineEnum::Value(t)
    }
}

impl<T> Into<u8> for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn into(self) -> u8 {
        match self {
            MachineEnum::Value(v) => v.into(),
            MachineEnum::Unknown(v) => v,
        }
    }
}

impl<T> Into<Option<T>> for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn into(self) -> Option<T> {
        match self {
            MachineEnum::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl<T: Debug> Debug for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(t) => t.fmt(f),
            Self::Unknown(v) => format!("Unknown({})", v).fmt(f),
        }
    }
}

impl<T: PartialEq> PartialEq<T> for MachineEnum<T>
where
    T: TryFrom<u8> + Copy,
    u8: From<T>,
{
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Value(t) => t.eq(other),
            Self::Unknown(_v) => false,
        }
    }
}
