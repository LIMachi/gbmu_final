#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Reg { ST, A, F, AF, B, C, BC, D, E, DE, H, L, HL, SP, PC }

pub trait Flags {
    fn zero(&self) -> bool;
    fn sub(&self) -> bool;
    fn half(&self) -> bool;
    fn carry(&self) -> bool;
}

impl Flags for u8 {
    fn zero(&self) -> bool { (self >> 7) & 1 == 1 }
    fn sub(&self) -> bool { (self >> 6) & 1 == 1 }
    fn half(&self) -> bool { (self >> 5) & 1 == 1 }
    fn carry(&self) -> bool { (self >> 4) & 1 == 1 }
}

pub mod regs {
    pub const A: u8 = 1;
    pub const F: u8 = 2;
    pub const B: u8 = 3;
    pub const C: u8 = 4;
    pub const D: u8 = 5;
    pub const E: u8 = 6;
    pub const H: u8 = 7;
    pub const L: u8 = 8;
    pub const AF: u8 = 9;
    pub const BC: u8 = 10;
    pub const DE: u8 = 11;
    pub const HL: u8 = 12;
    pub const SP: u8 = 13;
    pub const PC: u8 = 14;

    pub const ST: u8 = 0;
}

impl From<u8> for Reg {
    fn from(value: u8) -> Self {
        match value {
            0 => Reg::ST,
            1 => Reg::A,
            2 => Reg::F,
            3 => Reg::B,
            4 => Reg::C,
            5 => Reg::D,
            6 => Reg::E,
            7 => Reg::H,
            8 => Reg::L,
            9 => Reg::AF,
            10 => Reg::BC,
            11 => Reg::HL,
            13 => Reg::SP,
            14 => Reg::PC,
            _ => unreachable!()
        }
    }
}
