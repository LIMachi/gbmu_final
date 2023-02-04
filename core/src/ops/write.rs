use super::*;

fn a(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::A, value);
    CONTINUE
}

fn b(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::B, value);
    CONTINUE
}

fn c(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::C, value);
    CONTINUE
}

fn d(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::D, value);
    CONTINUE
}

fn e(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::E, value);
    CONTINUE
}

fn h(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::H, value);
    CONTINUE
}

fn l(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::L, value);
    CONTINUE
}

fn af(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::AF, value);
    CONTINUE
}

fn bc(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::BC, value);
    CONTINUE
}

fn de(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::DE, value);
    CONTINUE
}

fn hl(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::HL, value);
    CONTINUE
}

fn sp(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::SP, value);
    CONTINUE
}

fn pc(state: &mut State) -> Flow {
    let value = state.pop();
    state.set_register(Reg::PC, value);
    CONTINUE
}
