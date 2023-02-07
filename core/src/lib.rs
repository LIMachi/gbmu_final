extern crate core;

mod cpu;
mod ops;
mod registers;
mod decode;

use std::fmt;
use std::fmt::{Formatter, LowerHex, Write};
use log::{info, warn};

use registers::*;
use shared::cpu::{Value, Opcode, CBOpcode, Reg};
pub use cpu::Cpu;

pub trait Bus {
    fn status(&self) -> MemStatus;
    fn update(&mut self, status: MemStatus);
    fn tick(&mut self);
    fn get_range(&self, start: u16, len: u16) -> Vec<u8>;
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

impl MemStatus {
    pub fn read(&mut self) -> u8 {
        let v = match self {
            MemStatus::Read(v) => *v,
            _ => panic!("unexpected mem read")
        };
        *self = MemStatus::Ready;
        v
    }

    pub fn write(&mut self) -> u16 {
        let addr = match self {
            MemStatus::Write(addr) => { *addr },
            _ => panic!("unexpected mem write")
        };
        *self = MemStatus::Ready;
        addr
    }

    pub fn req_read(&mut self, addr: u16) {
        *self = match self {
            MemStatus::Idle | MemStatus::Ready => { MemStatus::ReqRead(addr) },
            _ => panic!("invalid state")
        }
    }

    pub fn req_write(&mut self, addr: u16) {
        *self = match self {
            MemStatus::Idle | MemStatus::Ready => { MemStatus::ReqWrite(addr) },
            _ => panic!("invalid state")
        }
    }
}

pub struct State<'a> {
    mem: MemStatus,
    bus: &'a mut dyn Bus,
    regs: &'a mut Registers,
    cache: &'a mut Vec<Value>
}

impl<'a> Drop for State<'a> {
    fn drop(&mut self) {
        match self.mem {
            MemStatus::ReqRead(_) | MemStatus::ReqWrite(_) => { },
            e => {
                if e != MemStatus::Idle && e!= MemStatus::Ready { warn!("{e:?} I/O result wasn't used this cycle") };
                info!("req read pc: {:x?}", self.regs.pc());
                self.mem = MemStatus::ReqRead(self.regs.pc());
            },
        };
        self.bus.update(self.mem);
    }
}

#[derive(Default)]
pub struct Flags {
    zero: Option<bool>,
    half: Option<bool>,
    sub: Option<bool>,
    carry: Option<bool>
}

impl Flags {
    pub fn c() -> Self { Self::default().set_carry(true) }
    pub fn nc() -> Self { Self::default().set_carry(false) }
    pub fn z() -> Self { Self::default().set_zero(true) }
    pub fn nz() -> Self { Self::default().set_zero(false) }

    pub fn set_zero(mut self, z: bool) -> Self { self.zero = Some(z); self }
    pub fn set_carry(mut self, c: bool) -> Self { self.carry = Some(c); self }
    pub fn set_half(mut self, h: bool) -> Self { self.half = Some(h); self }
    pub fn set_sub(mut self, s: bool) -> Self { self.sub = Some(s); self }

    pub fn carry(&self) -> bool { self.carry.expect("unexpected carry flag read") }
    pub fn half(&self) -> bool { self.half.expect("unexpected half flag read") }
    pub fn sub(&self) -> bool { self.sub.expect("unexpected sub flag read") }
    pub fn zero(&self) -> bool { self.zero.expect("unexpected zero flag read") }
}

impl<'a> State<'a> {
    pub fn new(bus: &'a mut dyn Bus, regs: &'a mut Registers, stack: &'a mut Vec<Value>) -> Self {
        Self {
            mem: bus.status(),
            bus,
            regs,
            cache: stack
        }
    }

    pub fn read(&mut self) -> u8 {
        self.mem.read()
    }

    pub fn write(&mut self, value: Value) {
        let addr = self.mem.write();
        // self.bus.write(addr, value);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn peek(&self) -> Option<Value> {
        self.cache.get(0).map(|x| *x)
    }

    pub fn push(&mut self, value: Value) {
        self.cache.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.cache.pop().expect("stack empty")
    }

    pub fn register(&self, register: Reg) -> Value {
        self.regs.read(register)
    }

    pub fn set_register(&mut self, register: Reg, value: Value) {
        match (register, value) {
            (Reg::A, Value::U8(v)) => self.regs.set_a(v),
            (Reg::B, Value::U8(v)) => self.regs.set_b(v),
            (Reg::C, Value::U8(v)) => self.regs.set_c(v),
            (Reg::D, Value::U8(v)) => self.regs.set_d(v),
            (Reg::E, Value::U8(v)) => self.regs.set_e(v),
            (Reg::H, Value::U8(v)) => self.regs.set_h(v),
            (Reg::L, Value::U8(v)) => self.regs.set_l(v),
            (Reg::AF, Value::U16(v)) => self.regs.set_af(v),
            (Reg::BC, Value::U16(v)) => self.regs.set_bc(v),
            (Reg::DE, Value::U16(v)) => self.regs.set_de(v),
            (Reg::HL, Value::U16(v)) => self.regs.set_hl(v),
            (Reg::SP, Value::U16(v)) => self.regs.set_sp(v),
            (Reg::PC, Value::U16(v)) => self.regs.set_pc(v),
            _ => panic!("reg and value size mismatch")
        }
    }

    pub fn set_flags(&mut self, flags: Flags) {
        if let Some(z) = flags.zero { self.regs.set_zero(z); }
        if let Some(h) = flags.half { self.regs.set_half(h); }
        if let Some(c) = flags.carry { self.regs.set_carry(c); }
        if let Some(s) = flags.sub { self.regs.set_sub(s); }
    }

    pub fn flags(&self) -> Flags {
        Flags::default()
            .set_carry(self.regs.carry())
            .set_sub(self.regs.sub())
            .set_half(self.regs.half())
            .set_zero(self.regs.zero())
    }

    pub fn req_read(&mut self, addr: u16) {
        self.mem.req_read(addr);
    }
    pub fn req_write(&mut self, addr: u16) {
        self.mem.req_write(addr);
    }
}
