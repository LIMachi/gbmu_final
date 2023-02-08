use super::*;

pub fn pc(state: &mut State) -> Flow {
    if let Value::U16(v) = state.register(Reg::PC) {
        state.set_register(Reg::PC, Value::U16(v - 1));
    }
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    if let Value::U16(v) = state.register(Reg::PC) {
        state.set_register(Reg::PC, Value::U16(v - 1));
    }
    CONTINUE
}

pub fn dec(state: &mut State) -> Flow {
    let v = state.pop().u8() - 1;
    state.push(v.into());
    state.flags()
        .set_zero(v == 0)
        .set_sub(true)
        .set_half((v & 0x10) != 0);
    CONTINUE
}

pub fn dec16(state: &mut State) -> Flow {
    let v = state.pop().u16() - 1;
    state.push(v.into());
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    let hl = state.register(Reg::HL).u16() - 1;
    state.set_register(Reg::HL, hl.into());
    CONTINUE
}
