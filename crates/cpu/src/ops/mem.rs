use log::warn;
use super::*;

pub fn cb(state: &mut State) -> Flow {
    *state.prefix = true;
    CONTINUE
}

/*pub fn read_pc(state: &mut State) -> Flow {
    if let Value::U16(addr) = state.register(Reg::PC) {
        state.req_read(addr);
    }
    CONTINUE
}

pub fn read_sp(state: &mut State) -> Flow {
    let sp = state.register(Reg::SP).u16();
    state.req_read(sp);
    CONTINUE
}
*/
pub fn write_sp(state: &mut State) -> Flow {
    let sp = state.register(Reg::SP).u16();
    state.req_write(sp);
    CONTINUE
}

pub fn req_read(state: &mut State) -> Flow {
    let addr = state.pop().u16();
    state.req_read(addr);
    CONTINUE
}

pub fn req_write(state: &mut State) -> Flow {
    let addr = state.pop().u16();
    state.req_write(addr);
    CONTINUE
}

/// LDH: take u8 from stack, add 0xFF00, req read
pub fn req_read_u8(state: &mut State) -> Flow {
    let addr = state.pop().u8();
    let addr = u16::from_le_bytes([addr, 0xFF]);
    state.req_read(addr);
    CONTINUE
}

/// LDH: take u8 from stack, add 0xFF00, req write
pub fn req_write_u8(state: &mut State) -> Flow {
    let addr = state.pop().u8();
    state.req_write(u16::from_le_bytes([addr, 0xFF]));
    CONTINUE
}

pub fn split(state: &mut State) -> Flow {
    let addr = state.pop().u16();
    let [low, high] = addr.to_le_bytes();
    state.push(low);
    state.push(high);
    CONTINUE
}

pub fn merge(state: &mut State) -> Flow {
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
