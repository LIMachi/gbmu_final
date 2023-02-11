use shared::cpu::{Reg, Value};

const HIGH: usize = 1;
const LOW: usize = 0;

#[derive(Clone)]
pub struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16
}

impl Registers {
    pub fn read(&self, reg: Reg) -> Value {
        match reg {
            Reg::A => Value::U8(self.a),
            Reg::F => Value::U8(self.f),
            Reg::B => Value::U8(self.b),
            Reg::C => Value::U8(self.c),
            Reg::D => Value::U8(self.d),
            Reg::E => Value::U8(self.e),
            Reg::H => Value::U8(self.h),
            Reg::L => Value::U8(self.l),
            Reg::AF => Value::U16(self.af()),
            Reg::BC => Value::U16(self.bc()),
            Reg::DE => Value::U16(self.de()),
            Reg::HL => Value::U16(self.hl()),
            Reg::SP => Value::U16(self.sp),
            Reg::PC => Value::U16(self.pc),
            e => panic!("invalid read {:?}", e)
        }
    }
    pub fn a(&self) -> u8 { self.a }
    pub fn f(&self) -> u8 { self.f }
    pub fn b(&self) -> u8 { self.b }
    pub fn c(&self) -> u8 { self.c }
    pub fn d(&self) -> u8 { self.d }
    pub fn e(&self) -> u8 { self.e }
    pub fn h(&self) -> u8 { self.h }
    pub fn l(&self) -> u8 { self.l }
    pub fn af(&self) -> u16 { u16::from_le_bytes([self.f, self.a]) }
    pub fn bc(&self) -> u16 { u16::from_le_bytes([self.c, self.b]) }
    pub fn de(&self) -> u16 { u16::from_le_bytes([self.e, self.d]) }
    pub fn hl(&self) -> u16 { u16::from_le_bytes([self.l, self.h]) }
    pub fn sp(&self) -> u16 { self.sp }
    pub fn pc(&self) -> u16 { self.pc }

    pub fn zero(&self) -> bool { (self.f >> 7) & 1 == 1 }
    pub fn sub(&self) -> bool { (self.f >> 6) & 1 == 1 }
    pub fn half(&self) -> bool { (self.f >> 5) & 1 == 1 }
    pub fn carry(&self) -> bool { (self.f >> 4) & 1 == 1 }

    pub fn set_a(&mut self, value: u8) { self.a = value; }
    pub fn set_b(&mut self, value: u8) { self.b = value; }
    pub fn set_c(&mut self, value: u8) { self.c = value; }
    pub fn set_d(&mut self, value: u8) { self.d = value; }
    pub fn set_e(&mut self, value: u8) { self.e = value; }
    pub fn set_h(&mut self, value: u8) { self.h = value; }
    pub fn set_l(&mut self, value: u8) { self.l = value; }

    pub fn set_af(&mut self, value: u16) { let [f, a] = value.to_le_bytes(); self.a = a; self.f = f; }
    pub fn set_bc(&mut self, value: u16) { let [c, b] = value.to_le_bytes(); self.b = b; self.c = c; }
    pub fn set_de(&mut self, value: u16) { let [e, d] = value.to_le_bytes(); self.d = d; self.e = e; }
    pub fn set_hl(&mut self, value: u16) { let [l, h] = value.to_le_bytes(); self.l = l; self.h = h; }
    pub fn set_pc(&mut self, value: u16) { self.pc = value; }
    pub fn set_sp(&mut self, value: u16) { self.sp = value; }

    pub fn set_zero(&mut self, value: bool) { self.f = self.f & !0x80 | ((value as u8) << 7); }
    pub fn set_sub(&mut self, value: bool) { self.f = self.f & !0x40 | ((value as u8) << 6); }
    pub fn set_half(&mut self, value: bool) { self.f = self.f & !0x20 | ((value as u8) << 5); }
    pub fn set_carry(&mut self, value: bool) { self.f = self.f & !0x10 | ((value as u8) << 4); }
}

impl Registers {
    pub const GB: Registers = Registers {
        a: 0x01,
        f: 0xB0,
        b: 0x13,
        c: 0x00,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
        sp: 0xFFFE,
        pc: 0x0100
    };

    pub const GBC: Registers = Registers {
        a: 0x11,
        f: 0xB0,
        b: 0x13,
        c: 0x00,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
        sp: 0xFFFE,
        pc: 0x0100
    };
}
