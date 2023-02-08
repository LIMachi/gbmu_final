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
use crate::ops::alu::add;

pub trait Bus {
    fn status(&self) -> MemStatus;
    fn update(&mut self, status: MemStatus);
    fn tick(&mut self);
    fn get_range(&self, start: u16, len: u16) -> Vec<u8>;
    fn write(&mut self, addr: u16, value: u8);
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
            MemStatus::Read(_) => {
                warn!("unused read");
                MemStatus::ReqWrite(addr)
            },
            s => panic!("invalid state {s:?}")
        }
    }
}
#[derive(Default, Copy, Clone)]
pub struct Flags {
    zero: bool,
    half: bool,
    sub: bool,
    carry: bool
}

pub struct State<'a> {
    mem: MemStatus,
    bus: &'a mut dyn Bus,
    regs: &'a mut Registers,
    cache: &'a mut Vec<Value>,
    flags: Option<Flags>,
    prefix: &'a mut bool
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
        if let Some(flags) = self.flags {
            self.regs.set_zero(flags.zero);
            self.regs.set_sub(flags.sub);
            self.regs.set_half(flags.half);
            self.regs.set_carry(flags.carry);
        }
        self.bus.update(self.mem);
    }
}


impl Flags {
    pub fn get(r: &Registers) -> Self {
        Self {
            zero: r.zero(),
            half: r.half(),
            sub: r.sub(),
            carry: r.carry()
        }
    }

    pub fn set_zero(&mut self, z: bool) -> &mut Self { self.zero = z; self }
    pub fn set_carry(&mut self, c: bool) -> &mut Self { self.carry = c; self }
    pub fn set_half(&mut self, h: bool) -> &mut Self { self.half = h; self }
    pub fn set_sub(&mut self, s: bool) -> &mut Self { self.sub = s; self }

    pub fn carry(&self) -> bool { self.carry }
    pub fn half(&self) -> bool { self.half }
    pub fn sub(&self) -> bool { self.sub }
    pub fn zero(&self) -> bool { self.zero }
}

impl<'a> State<'a> {
    pub fn new(bus: &'a mut dyn Bus, (regs, cache, prefix): (&'a mut Registers, &'a mut Vec<Value>, &'a mut bool)) -> Self {
        Self {
            mem: bus.status(),
            bus,
            flags: None,
            regs,
            cache,
            prefix
        }
    }

    pub fn read(&mut self) -> u8 {
        self.mem.read()
    }

    pub fn write(&mut self, value: Value) {
        let addr = self.mem.write();
        match value {
            Value::U8(v) => self.bus.write(addr,v),
            Value::U16(v) => {
                let [low, high] = v.to_le_bytes();
                self.bus.write(addr, low);
                self.bus.write(addr + 1, high);
            }
        }
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

    /// Init flags
    pub fn flags(&mut self) -> &mut Flags {
        if self.flags.is_none() {
            self.flags = Some(Flags::get(self.regs));
        }
        self.flags.as_mut().unwrap()
    }

    pub fn req_read(&mut self, addr: u16) {
        self.mem.req_read(addr);
    }
    pub fn req_write(&mut self, addr: u16) {
        self.mem.req_write(addr);
    }
}
