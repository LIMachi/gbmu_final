
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Reg { A, F, AF, B, C, BC, D, E, DE, H, L, HL, SP, PC }
