use log::warn;
use super::*;

pub fn read_pc(state: &mut State) -> Flow {
    if let Value::U16(addr) = state.register(Reg::PC) {
        state.req_read(addr);
    }
    CONTINUE
}

pub fn read_sp(state: &mut State) -> Flow {
    if let Value::U16(addr) = state.register(Reg::PC) {
        state.req_read(addr);
    }
    CONTINUE
}

pub fn read(state: &mut State) -> Flow {
    unimplemented!();
    CONTINUE
}

pub fn write(state: &mut State) -> Flow {
    unimplemented!();
    CONTINUE
}

pub fn push16(state: &mut State) -> Flow {
    if let Value::U16(_) = state.peek().expect("empty cache") {
        warn!("unexpected push16 whith u16 in the cache");
        return CONTINUE;
    };
    match (state.pop(), state.pop()) {
        (Value::U8(high), Value::U8(low)) => state.push(Value::U16(u16::from_le_bytes([low, high]))),
        _ => panic!("expected double u8, found u8/u16 on the stack")
    };
    CONTINUE
}
