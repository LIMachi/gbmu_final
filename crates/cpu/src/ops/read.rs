use super::*;

pub fn a(state: &mut State) -> Flow {
    let v = state.register(Reg::A);
    state.push(v);
    CONTINUE
}

pub fn b(state: &mut State) -> Flow {
    let v = state.register(Reg::B);
    state.push(v);
    CONTINUE
}

pub fn c(state: &mut State) -> Flow {
    let v = state.register(Reg::C);
    state.push(v);
    CONTINUE
}

pub fn d(state: &mut State) -> Flow {
    let v = state.register(Reg::D);
    state.push(v);
    CONTINUE
}

pub fn e(state: &mut State) -> Flow {
    let v = state.register(Reg::E);
    state.push(v);
    CONTINUE
}

pub fn f(state: &mut State) -> Flow {
    let v = state.register(Reg::F);
    state.push(v);
    CONTINUE
}

pub fn h(state: &mut State) -> Flow {
    let v = state.register(Reg::H);
    state.push(v);
    CONTINUE
}

pub fn l(state: &mut State) -> Flow {
    let v = state.register(Reg::L);
    state.push(v);
    CONTINUE
}

/*pub fn af(state: &mut State) -> Flow {
    let v = state.register(Reg::AF);
    state.push(v);
    CONTINUE
}*/

pub fn bc(state: &mut State) -> Flow {
    let v = state.register(Reg::BC);
    state.push(v);
    CONTINUE
}

pub fn de(state: &mut State) -> Flow {
    let v = state.register(Reg::DE);
    state.push(v);
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    let v = state.register(Reg::HL);
    state.push(v);
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    let v = state.register(Reg::SP);
    state.push(v);
    CONTINUE
}

pub fn pc(state: &mut State) -> Flow {
    let v = state.register(Reg::PC);
    state.push(v);
    CONTINUE
}

pub fn mem(state: &mut State) -> Flow {
    let v = state.read();
    state.push(Value::U8(v));
    CONTINUE
}

fn fixed<const N: u8>(state: &mut State) -> Flow {
    state.push(Value::U16(u16::from_le_bytes([N, 0])));
    CONTINUE
}

pub const FIXED_0: Op = fixed::<0x0>;
pub const FIXED_10: Op = fixed::<0x10>;
pub const FIXED_20: Op = fixed::<0x20>;
pub const FIXED_30: Op = fixed::<0x30>;
pub const FIXED_8: Op = fixed::<0x8>;
pub const FIXED_18: Op = fixed::<0x18>;
pub const FIXED_28: Op = fixed::<0x28>;
pub const FIXED_38: Op = fixed::<0x38>;

pub const INT_VBLANK: Op = fixed::<0x40>;
pub const INT_STAT  : Op = fixed::<0x48>;
pub const INT_TIMER : Op = fixed::<0x50>;
pub const INT_SERIAL: Op = fixed::<0x58>;
pub const INT_JOY   : Op = fixed::<0x60>;
