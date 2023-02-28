use super::*;

pub fn z(state: &mut State) -> Flow {
    if state.flags().zero() { CONTINUE } else { BREAK }
}

pub fn nz(state: &mut State) -> Flow {
    if !state.flags().zero() { CONTINUE } else { BREAK }
}

pub fn c(state: &mut State) -> Flow {
    if state.flags().carry() { CONTINUE } else { BREAK }
}

pub fn nc(state: &mut State) -> Flow {
    if !state.flags().carry() { CONTINUE } else { BREAK }
}
