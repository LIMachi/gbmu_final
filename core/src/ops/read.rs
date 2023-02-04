use super::*;

fn a(state: &mut State) -> Flow {
    state.push(state.register(Reg::A));
    CONTINUE
}

fn b(state: &mut State) -> Flow {
    state.push(state.register(Reg::B));
    CONTINUE
}

fn c(state: &mut State) -> Flow {
    state.push(state.register(Reg::C));
    CONTINUE
}

fn d(state: &mut State) -> Flow {
    state.push(state.register(Reg::D));
    CONTINUE
}

fn e(state: &mut State) -> Flow {
    state.push(state.register(Reg::E));
    CONTINUE
}

fn h(state: &mut State) -> Flow {
    state.push(state.register(Reg::H));
    CONTINUE
}

fn l(state: &mut State) -> Flow {
    state.push(state.register(Reg::L));
    CONTINUE
}

fn af(state: &mut State) -> Flow {
    state.push(state.register(Reg::AF));
    CONTINUE
}

fn bc(state: &mut State) -> Flow {
    state.push(state.register(Reg::BC));
    CONTINUE
}

fn de(state: &mut State) -> Flow {
    state.push(state.register(Reg::DE));
    CONTINUE
}

fn hl(state: &mut State) -> Flow {
    state.push(state.register(Reg::HL));
    CONTINUE
}

fn sp(state: &mut State) -> Flow {
    state.push(state.register(Reg::SP));
    CONTINUE
}

fn pc(state: &mut State) -> Flow {
    state.push(state.register(Reg::PC));
    CONTINUE
}
