use super::*;

pub fn a(state: &mut State) -> Flow {
    state.push(state.register(Reg::A));
    CONTINUE
}

pub fn b(state: &mut State) -> Flow {
    state.push(state.register(Reg::B));
    CONTINUE
}

pub fn c(state: &mut State) -> Flow {
    state.push(state.register(Reg::C));
    CONTINUE
}

pub fn d(state: &mut State) -> Flow {
    state.push(state.register(Reg::D));
    CONTINUE
}

pub fn e(state: &mut State) -> Flow {
    state.push(state.register(Reg::E));
    CONTINUE
}

pub fn f(state: &mut State) -> Flow {
    state.push(state.register(Reg::F));
    CONTINUE
}

pub fn h(state: &mut State) -> Flow {
    state.push(state.register(Reg::H));
    CONTINUE
}

pub fn l(state: &mut State) -> Flow {
    state.push(state.register(Reg::L));
    CONTINUE
}

pub fn af(state: &mut State) -> Flow {
    state.push(state.register(Reg::AF));
    CONTINUE
}

pub fn bc(state: &mut State) -> Flow {
    state.push(state.register(Reg::BC));
    CONTINUE
}

pub fn de(state: &mut State) -> Flow {
    state.push(state.register(Reg::DE));
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    state.push(state.register(Reg::HL));
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    state.push(state.register(Reg::SP));
    CONTINUE
}

pub fn pc(state: &mut State) -> Flow {
    state.push(state.register(Reg::PC));
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
