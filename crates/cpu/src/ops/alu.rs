use std::ops::{BitAnd, BitOr, BitXor};
use super::*;

pub fn cmp(state: &mut State) -> Flow {
    let n = state.pop().u8();
    let a = state.register(Reg::A).u8();
    let (r, o) = a.overflowing_sub(n);
    state.flags()
        .set_zero(r == 0)
        .set_sub(true)
        .set_half((a & 0xF) < (n & 0xF))
        .set_carry(o);
    CONTINUE
}

pub fn sub(state: &mut State) -> Flow {
    let n = state.pop().u8();
    let a = state.register(Reg::A).u8();
    let (r, o) = a.overflowing_sub(n);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(true)
        .set_half((a & 0xF) < (n & 0xF))
        .set_carry(o);
    CONTINUE
}

pub fn sbc(state: &mut State) -> Flow {
    let n = state.pop().u8();
    let cr = state.flags().carry() as u8;
    let a = state.register(Reg::A).u8();
    let (r, o) = a.overflowing_sub(n);
    let (r, c) = r.overflowing_sub(cr);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(true)
        .set_half((a & 0xF) < (n & 0xF) + cr)
        .set_carry(o || c);
    CONTINUE
}

pub fn add(state: &mut State) -> Flow {
    let n = state.pop().u8();
    let a = state.register(Reg::A).u8();
    let (r, o) = a.overflowing_add(n);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(false)
        .set_half(((a & 0xF) + (n & 0xF)) & 0x10 != 0)
        .set_carry(o);
    CONTINUE
}

pub fn add_sp(state: &mut State) -> Flow {
    let n = state.pop().u8() as i8 as u16;
    let r = state.register(Reg::SP).u16();
    let res = r.wrapping_add(n);
    state.flags()
        .set_zero(false)
        .set_sub(false)
        .set_half((n & 0xF) + (r & 0xF) > 0xF)
        .set_carry((n & 0xFF) + (r & 0xFF) > 0xFF);
    state.push(res);
    CONTINUE
}

/// technically, this bypasses ALU. whatever.
pub fn add_hl(state: &mut State) -> Flow {
    let hl = state.register(Reg::HL).u16();
    let n = state.pop().u16();
    let (r, c) = hl.overflowing_add(n);
    state.set_register(Reg::HL, r);
    state.flags()
        .set_sub(false)
        .set_half(((hl & 0xFFF) + (n & 0xFFF)) & 0x1000 != 0)
        .set_carry(c);
    CONTINUE
}

pub fn adc(state: &mut State) -> Flow {
    let n = state.pop().u8();
    let cr = state.flags().carry() as u8;
    let a = state.register(Reg::A).u8();
    let (r, o) = a.overflowing_add(n);
    let (r, c) = r.overflowing_add(cr);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(false)
        .set_half(((a & 0xF) + (n & 0xF) + cr) & 0x10 != 0)
        .set_carry(o || c);
    CONTINUE
}

pub fn xor(state: &mut State) -> Flow {
    let a = state.register(Reg::A).u8();
    let o = state.pop().u8();
    let r = a.bitxor(o);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(false)
        .set_half(false)
        .set_carry(false);
    CONTINUE
}

pub fn or(state: &mut State) -> Flow {
    let a = state.register(Reg::A).u8();
    let o = state.pop().u8();
    let r = a.bitor(o);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(false)
        .set_half(false)
        .set_carry(false);
    CONTINUE
}

pub fn and(state: &mut State) -> Flow {
    let a = state.register(Reg::A).u8();
    let o = state.pop().u8();
    let r = a.bitand(o);
    state.set_register(Reg::A, r);
    state.flags().set_zero(r == 0)
        .set_sub(false)
        .set_half(true)
        .set_carry(false);
    CONTINUE
}

pub fn cpl(state: &mut State) -> Flow {
    let a = !state.register(Reg::A).u8();
    state.flags()
        .set_sub(true)
        .set_half(true);
    state.set_register(Reg::A, a);
    CONTINUE
}

/// dark magic ! (transforms hex to bcd in reg a)
pub fn daa(state: &mut State) -> Flow {
    let (mut a, mut c) = (state.register(Reg::A).u8(), false);
    match state.flags().sub() {
        false => {
            if state.flags().carry() || a > 0x99 { a = a.wrapping_add(0x60); c = true; }
            if state.flags().half() || (a & 0xF) > 0x9 { a += 0x6; }
        },
        true => {
            if state.flags().carry() { a = a.wrapping_sub(0x60); c = true; }
            if state.flags().half() { a = a.wrapping_sub(0x6); }
        }
    }
    state.set_register(Reg::A, Value::U8(a));
    state.flags().set_carry(c)
        .set_half(false)
        .set_zero(a == 0);
    CONTINUE
}

pub fn scf(state: &mut State) -> Flow {
    state.flags()
        .set_sub(false)
        .set_half(false)
        .set_carry(true);
    CONTINUE
}

pub fn ccf(state: &mut State) -> Flow {
    let c = state.flags().carry;
    state.flags()
        .set_sub(false)
        .set_half(false)
        .set_carry(!c);
    CONTINUE
}
