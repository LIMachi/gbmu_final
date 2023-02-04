use super::Value;

const HIGH: usize = 1;
const LOW: usize = 0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reg { A, F, AF, B, C, BC, D, E, DE, H, L, HL, SP, PC }

pub struct Registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16
}


impl Registers {
    pub fn read(&self, reg: Reg) -> Value {
        match reg {
            Reg::A => Value::U8(self.a()),
            Reg::B => Value::U8(self.b()),
            Reg::C => Value::U8(self.c()),
            Reg::D => Value::U8(self.d()),
            Reg::E => Value::U8(self.e()),
            Reg::H => Value::U8(self.h()),
            Reg::L => Value::U8(self.l()),
            Reg::AF => Value::U16(self.af),
            Reg::BC => Value::U16(self.bc),
            Reg::DE => Value::U16(self.de),
            Reg::HL => Value::U16(self.hl),
            Reg::SP => Value::U16(self.sp),
            Reg::PC => Value::U16(self.pc),
            e => panic!("invalid read {:?}", e)
        }
    }

    pub fn a(&self) -> u8 { self.af.to_le_bytes()[HIGH] }
    pub fn b(&self) -> u8 { self.bc.to_le_bytes()[HIGH] }
    pub fn c(&self) -> u8 { self.bc.to_le_bytes()[LOW] }
    pub fn d(&self) -> u8 { self.de.to_le_bytes()[HIGH] }
    pub fn e(&self) -> u8 { self.de.to_le_bytes()[LOW] }
    pub fn h(&self) -> u8 { self.hl.to_le_bytes()[HIGH] }
    pub fn l(&self) -> u8 { self.hl.to_le_bytes()[LOW] }

    #[inline]
    fn f(&self) -> u8 { self.af.to_le_bytes()[LOW] }

    pub fn zero(&self) -> bool { (self.f() >> 7) & 1 == 1 }
    pub fn sub(&self) -> bool { (self.f() >> 6) & 1 == 1 }
    pub fn half(&self) -> bool { (self.f() >> 5) & 1 == 1 }
    pub fn carry(&self) -> bool { (self.f() >> 4) & 1 == 1 }

    pub fn set_a(&mut self, value: u8) { self.af = u16::from_le_bytes([self.f(), value]); }
    pub fn set_b(&mut self, value: u8) { self.bc = u16::from_le_bytes([self.c(), value]); }
    pub fn set_c(&mut self, value: u8) { self.bc = u16::from_le_bytes([value, self.b()]); }
    pub fn set_d(&mut self, value: u8) { self.de = u16::from_le_bytes([self.d(), value]); }
    pub fn set_e(&mut self, value: u8) { self.de = u16::from_le_bytes([value, self.e()]); }
    pub fn set_h(&mut self, value: u8) { self.hl = u16::from_le_bytes([self.h(), value]); }
    pub fn set_l(&mut self, value: u8) { self.hl = u16::from_le_bytes([value, self.l()]); }

    pub fn set_af(&mut self, value: u16) { self.af = value; }
    pub fn set_bc(&mut self, value: u16) { self.bc = value; }
    pub fn set_de(&mut self, value: u16) { self.bc = value; }
    pub fn set_hl(&mut self, value: u16) { self.de = value; }
    pub fn set_pc(&mut self, value: u16) { self.pc = value; }
    pub fn set_sp(&mut self, value: u16) { self.sp = value; }

    pub fn set_zero(&mut self, value: bool) { self.af = u16::from_le_bytes([self.f() & !0x80 | ((value as u8) << 7), self.a()]); }
    pub fn set_sub(&mut self, value: bool) { self.af = u16::from_le_bytes([self.f() & !0x40 | ((value as u8) << 6), self.a()]); }
    pub fn set_half(&mut self, value: bool) { self.af = u16::from_le_bytes([self.f() & !0x20 | ((value as u8) << 5), self.a()]); }
    pub fn set_carry(&mut self, value: bool) { self.af = u16::from_le_bytes([self.f() & !0x10 | ((value as u8) << 4), self.a()]); }
}
