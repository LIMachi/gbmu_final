use super::*;

pub fn bit<const N: usize, const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8() & (1 << N);
    state.flags()
        .set_zero(n == 0)
        .set_sub(false)
        .set_half(true);
    CONTINUE
}

pub fn set<const N: usize, const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8() | (1 << N);
    state.set_register(R, n);
    CONTINUE
}

pub fn res<const N: usize, const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8() & !(1 << N);
    state.set_register(R, n);
    CONTINUE
}

pub fn rr<const R: u8>(state: &mut State) -> Flow {
    let v = state.register(R).u8();
    let f = state.flags();
    let l = v & 0x1;
    let v = v >> 1 | (f.carry() as u8) << 7;
    f.set_carry(l == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, v);
    CONTINUE
}

pub fn rrc<const R: u8>(state: &mut State) -> Flow {
    let v = state.register(R).u8();
    let f = state.flags();
    let l = v & 0x1;
    let v = v >> 1 | l << 7;
    f.set_carry(l == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, v);
    CONTINUE
}

pub fn rl<const R: u8>(state: &mut State) -> Flow {
    let v = state.register(R).u8();
    let f = state.flags();
    let h = v >> 7;
    let v = v << 1 | (f.carry() as u8);
    f.set_carry(h == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, v);
    CONTINUE
}

pub fn rlc<const R: u8>(state: &mut State) -> Flow {
    let v = state.register(R).u8();
    let f = state.flags();
    let h = v >> 7;
    let v = v << 1 | h;
    f.set_carry(h == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, v);
    CONTINUE
}

pub fn sla<const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8();
    let f = state.flags().set_carry((n & 0x80) != 0);
    let n = (n << 1) & 0xFE;
    f.set_zero(n == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, n);
    CONTINUE
}

pub fn sra<const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8();
    let f = state.flags().set_carry((n & 0x1) != 0);
    let h = n & 0x80;
    let n = (n >> 1) | h;
    f.set_zero(n == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, n);
    CONTINUE
}

pub fn srl<const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8();
    let f = state.flags().set_carry((n & 0x1) != 0);
    let n = (n >> 1) & 0x7F;
    f.set_zero(n == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(R, n);
    CONTINUE
}

pub fn swap<const R: u8>(state: &mut State) -> Flow {
    let n = state.register(R).u8();
    let l = (n & 0xF) << 4;
    let n = l | (n >> 4);
    state.flags()
        .set_zero(n == 0)
        .set_half(false)
        .set_sub(false)
        .set_carry(false);
    state.set_register(R, n);
    CONTINUE
}

