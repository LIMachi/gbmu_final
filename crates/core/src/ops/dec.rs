use super::*;

pub fn pc(state: &mut State) -> Flow {
    let pc = state.register(Reg::PC).u16().wrapping_sub(1);
    state.set_register(Reg::PC, pc);
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    let sp = state.register(Reg::SP).u16() - 1;
    state.set_register(Reg::SP, sp);
    CONTINUE
}

pub fn dec(state: &mut State) -> Flow {
    let v = state.pop().u8().wrapping_sub(1);
    state.push(v);
    state.flags()
        .set_zero(v == 0)
        .set_sub(true)
        .set_half(v == 0xF);
    CONTINUE
}

pub fn dec16(state: &mut State) -> Flow {
    let v = state.pop().u16().wrapping_sub(1);
    state.push(v);
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    let hl = state.register(Reg::HL).u16().wrapping_sub(1);
    state.set_register(Reg::HL, hl);
    CONTINUE
}
