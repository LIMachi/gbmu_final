use super::*;

pub fn pc(state: &mut State) -> Flow {
    let pc = state.register(Reg::PC).u16() + 1;
    state.set_register(Reg::PC, pc);
    CONTINUE
}

pub fn jmp(state: &mut State) -> Flow {
    let mut pc = state.register(Reg::PC).u16();
    let e = state.pop().u8() as i8;
    pc = if e < 0 { pc - (-e) as u16 } else { pc + e as u16 };
    state.set_register(Reg::PC, pc);
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    let v = state.register(Reg::SP).u16() + 1;
    state.set_register(Reg::SP, v);
    CONTINUE
}

pub fn inc(state: &mut State) -> Flow {
    let v = state.pop().u8().wrapping_add(1);

    state.flags().set_zero(v == 0)
        .set_sub(false)
        .set_half((v & 0x10) != 0);
    state.push(v);
    CONTINUE
}

pub fn inc16(state: &mut State) -> Flow {
    let v = state.pop().u16().wrapping_add(1);
    state.push(v);
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    let hl = state.register(Reg::HL).u16() + 1;
    state.set_register(Reg::HL, hl);
    CONTINUE
}
