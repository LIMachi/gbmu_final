extern crate core;

mod cpu;
mod ops;
mod registers;
mod decode;

use registers::*;
use shared::cpu::{Value, Opcode, CBOpcode, Reg, MemStatus, Bus, regs};
pub use cpu::Cpu;
use crate::cpu::Mode;


trait RWStatus {
    fn read(&mut self) -> u8;
    fn write(&mut self) -> u16;
    fn req_read(&mut self, addr: u16);
    fn req_write(&mut self, addr: u16);
}

impl RWStatus for MemStatus {
    fn read(&mut self) -> u8 {
        match std::mem::replace(self, MemStatus::Ready) {
            MemStatus::Read(v) => v,
            _ => panic!("unexpected mem read")
        }
    }

    fn write(&mut self) -> u16 {
        match std::mem::replace(self, MemStatus::Ready) {
            MemStatus::Write(addr) => addr,
            _ => panic!("unexpected mem write")
        }
    }

    fn req_read(&mut self, addr: u16) {
        match std::mem::replace(self, MemStatus::ReqRead(addr)) {
            MemStatus::Idle | MemStatus::Ready | MemStatus::Read(_) => {},
            s => panic!("invalid state {s:?}")
        }
    }

    fn req_write(&mut self, addr: u16) {
        match std::mem::replace(self, MemStatus::ReqWrite(addr)) {
            MemStatus::Idle | MemStatus::Ready | MemStatus::Read(_) => {},
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
    prefix: &'a mut bool,
    ime: &'a mut bool,
    halt: &'a mut cpu::Mode,
}

impl<'a> Drop for State<'a> {
    fn drop(&mut self) {
        match self.mem {
            MemStatus::ReqRead(_) | MemStatus::ReqWrite(_) => { },
            e => {
                if e != MemStatus::Idle && e!= MemStatus::Ready { // warn!("{e:?} I/O result wasn't used this cycle")
                };
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
    pub(crate) fn new(bus: &'a mut dyn Bus, (regs, cache, prefix, ime, mode): (&'a mut Registers, &'a mut Vec<Value>, &'a mut bool, &'a mut bool, &'a mut cpu::Mode)) -> Self {
        Self {
            mem: bus.status(),
            bus,
            flags: None,
            regs,
            cache,
            prefix,
            ime,
            halt: mode
        }
    }

    pub fn read(&mut self) -> u8 {
        self.mem.read()
    }

    pub(crate) fn halt(&mut self) {
        *self.halt = Mode::Halt;
    }

    pub fn write(&mut self, value: u8) {
        self.bus.write(self.mem.write(), value);
    }

    pub fn write_low(&mut self, value: u8) {
        let addr = self.mem.write();
        self.req_write(addr + 1);
        self.bus.write(addr, value);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn peek(&self) -> Option<Value> {
        self.cache.get(0).map(|x| *x)
    }

    pub fn push<V: Into<Value>>(&mut self, value: V) {
        self.cache.push(value.into());
    }

    pub fn pop(&mut self) -> Value {
        self.cache.pop().expect("stack empty")
    }

    pub fn try_pop(&mut self) -> Option<Value> {
        self.cache.pop()
    }

    pub fn register<R: Into<Reg>>(&mut self, register: R) -> Value {
        match register.into() {
            Reg::ST => self.pop(),
            r => self.regs.read(r)
        }
    }

    pub fn set_register<R: Into<Reg>, V: Into<Value>>(&mut self, register: R, value: V) {
        match (register.into(), value.into()) {
            (Reg::A, Value::U8(v)) => self.regs.set_a(v),
            (Reg::B, Value::U8(v)) => self.regs.set_b(v),
            (Reg::C, Value::U8(v)) => self.regs.set_c(v),
            (Reg::D, Value::U8(v)) => self.regs.set_d(v),
            (Reg::E, Value::U8(v)) => self.regs.set_e(v),
            (Reg::F, Value::U8(v)) => self.regs.set_f(v & 0xF0),
            (Reg::H, Value::U8(v)) => self.regs.set_h(v),
            (Reg::L, Value::U8(v)) => self.regs.set_l(v),
            (Reg::AF, Value::U16(v)) => self.regs.set_af(v & 0xFFF0),
            (Reg::BC, Value::U16(v)) => self.regs.set_bc(v),
            (Reg::DE, Value::U16(v)) => self.regs.set_de(v),
            (Reg::HL, Value::U16(v)) => self.regs.set_hl(v),
            (Reg::SP, Value::U16(v)) => self.regs.set_sp(v),
            (Reg::PC, Value::U16(v)) => self.regs.set_pc(v),
            (Reg::ST, v) => self.push(v),
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
