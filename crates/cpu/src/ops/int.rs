use super::*;

pub fn on(state: &mut State) -> Flow {
    *state.ime = true;
    CONTINUE
}

pub fn off(state: &mut State) -> Flow {
    *state.ime = false;
    CONTINUE
}

pub fn stop(state: &mut State) -> Flow {
    *state.stop = 2050;
    state.stop();
    CONTINUE
}

pub fn halt(state: &mut State) -> Flow {
    state.halt();
    CONTINUE
}
