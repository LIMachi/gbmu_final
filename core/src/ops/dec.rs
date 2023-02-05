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

pub fn inc(state: &mut State) -> Flow {
    let v = state.pop();
    state.push(match v {
            Value::U8(v) => Value::U8(v - 1),
            Value::U16(_) => panic!("invalid dec")
        });
    CONTINUE
}

pub fn inc16(state: &mut State) -> Flow {
    let v = state.pop();
    state.push(match v {
            Value::U16(v) => Value::U16(v - 1),
            Value::U8(_) => panic!("invalid dec")
        });
    CONTINUE
}
