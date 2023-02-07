use std::fmt;
use std::fmt::{Formatter, LowerHex};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Value {
    U8(u8),
    U16(u16)
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::U8(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::U16(value)
    }
}

impl LowerHex for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::U8(v) => fmt::LowerHex::fmt(v, f),
            Value::U16(v) => fmt::LowerHex::fmt(v, f),
        }
    }
}
