use super::{Opcode, ops::Op};

pub fn decode(opcode: Opcode) -> &'static [&'static [Op]] {
    match opcode {
        Opcode::Nop => &[],
        o => unimplemented!("{o:?}")
    }
}
