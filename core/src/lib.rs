extern crate core;

mod cpu;
mod ops;
mod opcodes;
mod registers;

use registers::*;
use opcodes::*;
use cpu::Cpu;

enum Value {
    U8(u8),
    U16(u16)
}

// TODO
pub trait Bus {
    fn status(&self) -> MemStatus;
}

pub enum MemStatus {
    Read(u8),
    Write(u8),
    ReqRead(u16),
    ReqWrite(u16),
    Idle
}

impl MemStatus {
    pub fn read(&mut self) -> u8 {
        let v = match self {
            MemStatus::Read(v) => *v,
            _ => panic!("unexpected mem read")
        };
        *self = MemStatus::Idle;
        v
    }

    pub fn write(&mut self, value: u8) {
        match self {
            MemStatus::Idle => { *self = MemStatus::Write(value); },
            _ => panic!("unexpected mem write")
        }
    }
}

pub struct State<'a> {
    mem: MemStatus,
    mem_value: u8,
    regs: &'a mut Registers,
}

impl<'a> State<'a> {
    pub fn read(&self) -> u8 {
        self.mem_value
    }
}
