pub use super::opcodes::*;
pub use super::registers::{Reg, regs, Flags};
pub use super::value::Value;

pub trait Cpu {
    fn done(&self) -> bool;

    fn previous(&self) -> Opcode;
    fn register(&self, reg: Reg) -> Value;
}

pub trait Bus {
    fn status(&self) -> MemStatus;
    fn update(&mut self, status: MemStatus);
    fn get_range(&self, start: u16, len: u16) -> Vec<u8>;
    fn write(&mut self, addr: u16, value: u8);

    /// Bypasses read cycle
    /// CPU doesn't use this
    fn direct_read(&self, offset: u16) -> u8;
    fn int_reset(&mut self, bit: u8);
    fn int_set(&mut self, bit: u8);
    fn interrupt(&self) -> u8;
}

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum MemStatus {
    Read(u8),
    Write(u16),
    Ready,
    ReqRead(u16),
    ReqWrite(u16),
    Idle
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Read(u16, u8),
    Write(u16, u8)
}
