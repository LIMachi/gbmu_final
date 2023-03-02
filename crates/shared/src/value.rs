use std::fmt;
use std::fmt::{Formatter, LowerHex};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Value {
    U8(u8),
    U16(u16)
}

impl Value {
    pub fn u8(self) -> u8 {
        match self {
            Value::U8(n) => n,
            _ => { panic!("Wrong value size, expected U8") }
        }
    }
    pub fn u16(self) -> u16 {
        match self {
            Value::U16(n) => n,
            _ => { panic!("Wrong value size, expected U16") }
        }
    }
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
