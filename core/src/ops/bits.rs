use super::*;

pub fn bit<const N: usize>(state: &mut State) -> Flow {
    let n = state.pop().u8() & (1 << N);
    state.flags()
        .set_zero(n == 0)
        .set_sub(false)
        .set_half(true);
    CONTINUE
}

pub fn set<const N: usize>(state: &mut State) -> Flow {
    let n = state.pop().u8() | 1 << N;
    state.push(n.into());
    CONTINUE
}

pub fn res<const N: usize>(state: &mut State) -> Flow {
    let n = state.pop().u8() & !(1 << N);
    state.push(n.into());
    CONTINUE
}


pub fn rr(state: &mut State) -> Flow {
    let v = state.pop().u8();
    let f = state.flags();
    let l = v & 0x1;
    let v = v >> 1 | (f.carry() as u8) << 7;
    f.set_carry(l == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.push(v.into());
    CONTINUE
}

pub fn rrc(state: &mut State) -> Flow {
    let v = state.pop().u8();
    let f = state.flags();
    let l = v & 0x1;
    let v = v >> 1 | l << 7;
    f.set_carry(l == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.push(v.into());
    CONTINUE
}

pub fn rl(state: &mut State) -> Flow {
    let v = state.pop().u8();
    let f = state.flags();
    let h = v >> 7;
    let v = v << 1 | (f.carry() as u8);
    f.set_carry(h == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(Reg::A, v.into());
    CONTINUE
}

pub fn rlc(state: &mut State) -> Flow {
    let v = state.pop().u8();
    let f = state.flags();
    let h = v >> 7;
    let v = v << 1 | h;
    f.set_carry(h == 1);
    f.set_zero(v == 0)
        .set_sub(false)
        .set_half(false);
    state.set_register(Reg::A, v.into());
    CONTINUE
}

