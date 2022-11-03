use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// Helper trait that collects the requirements for a MachineEnum.
pub trait MachineEnumerable:
    TryFrom<u8> + Into<u8> + Copy + Debug + Eq + PartialEq + Ord + PartialOrd + Hash + Sized
{
}

/// Wraps a machine enumeration that may have unknown values.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub enum MachineEnum<T: MachineEnumerable> {
    Value(T),
    Unknown(u8),
}

impl<T> MachineEnum<T>
where
    T: MachineEnumerable,
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
    T: MachineEnumerable,
{
    fn from(t: T) -> Self {
        MachineEnum::Value(t)
    }
}

impl<T: MachineEnumerable> From<MachineEnum<T>> for u8 {
    fn from(v: MachineEnum<T>) -> Self {
        match v {
            MachineEnum::Value(v) => v.into(),
            MachineEnum::Unknown(v) => v,
        }
    }
}

impl<T: MachineEnumerable> From<MachineEnum<T>> for Option<T> {
    fn from(v: MachineEnum<T>) -> Self {
        match v {
            MachineEnum::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl<T: MachineEnumerable> Debug for MachineEnum<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(t) => t.fmt(f),
            Self::Unknown(v) => format!("Unknown({})", v).fmt(f),
        }
    }
}

impl<T: MachineEnumerable> PartialEq<T> for MachineEnum<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Value(t) => t.eq(other),
            Self::Unknown(_v) => false,
        }
    }
}

/// Represents a set of enum values, some potentially unknown.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SwitchSet<T: MachineEnumerable> {
    pub value: u16,
    phantom: PhantomData<T>,
}

impl<T: MachineEnumerable> SwitchSet<T> {
    pub fn of(input: &[T]) -> Self {
        let mut v = 0u16;
        for t in input {
            v |= 1 << <T as Into<u8>>::into(*t);
        }
        Self::from_u16(v)
    }

    pub fn empty() -> Self {
        Self::from_u16(0)
    }

    pub fn from_u16(v: u16) -> Self {
        SwitchSet {
            value: v,
            phantom: PhantomData::default(),
        }
    }

    pub fn set(&self) -> Vec<MachineEnum<T>> {
        // TODO: This should be an iterator
        let mut v = vec![];
        for i in 0..core::mem::size_of::<u16>() * 8 - 1 {
            if self.value & (1 << i) != 0 {
                let i = <u8>::try_from(i).expect("This should have fit in a u8");
                v.push(MachineEnum::<T>::decode(i));
            }
        }
        v
    }
}

impl<T: MachineEnumerable> std::fmt::Debug for SwitchSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value == 0 {
            f.write_str("(empty)")
        } else {
            let mut sep = "";
            for i in 0..core::mem::size_of::<u16>() * 8 - 1 {
                if self.value & (1 << i) != 0 {
                    let i = <u8>::try_from(i).expect("This should have fit in a u8");
                    f.write_fmt(format_args!("{}{:?}", sep, MachineEnum::<T>::decode(i)))?;
                    sep = " | ";
                }
            }
            Ok(())
        }
    }
}
