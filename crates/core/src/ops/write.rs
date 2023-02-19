use super::*;

pub fn a(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::A, value);
    CONTINUE
}

pub fn b(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::B, value);
    CONTINUE
}

pub fn c(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::C, value);
    CONTINUE
}

pub fn d(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::D, value);
    CONTINUE
}

pub fn e(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::E, value);
    CONTINUE
}

pub fn f(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::F, value);
    CONTINUE
}

pub fn h(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::H, value);
    CONTINUE
}

pub fn l(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::L, value);
    CONTINUE
}

pub fn af(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::AF, value);
    CONTINUE
}

pub fn bc(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::BC, value);
    CONTINUE
}

pub fn de(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::DE, value);
    CONTINUE
}

pub fn hl(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::HL, value);
    CONTINUE
}

pub fn sp(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::SP, value);
    CONTINUE
}

pub fn pc(state: &mut State) -> Flow {
    let v = state.pop();
    match v {
        Value::U16(v) => state.set_register(Reg::PC,Value::U16(v)),
        _ => panic!("expected u16 in cache.")
    };
    CONTINUE
}

pub fn mem(state: &mut State) -> Flow {
    let v = state.try_pop().expect(format!("failed to pop mem (pc {:#06X})", state.regs.pc()).as_str());
    state.write(v);
    CONTINUE
}
