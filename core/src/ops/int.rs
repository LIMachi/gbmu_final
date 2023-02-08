use super::*;

pub fn on(state: &mut State) -> Flow {
    CONTINUE
}

pub fn off(state: &mut State) -> Flow {
    CONTINUE
}

pub fn stop(state: &mut State) -> Flow {    //TODO pas fini
    info!("DON'T. STOP. ME. NOW.");
    CONTINUE
}

pub fn halt(state: &mut State) -> Flow {
    info!("HALT. F4");
    CONTINUE
}
