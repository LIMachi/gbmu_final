use crate::{Bus, Reg};
use super::{ops::{Flow, Op}, Value, State, Registers, Opcode, CBOpcode, decode::decode};

pub struct Cpu {
    instructions: Vec<Vec<Op>>,
    regs: Registers,
    stack: Vec<Value>
}

impl Cpu {

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let mut state = State::new(bus, &mut self.regs, &mut self.stack);
        if self.instructions.is_empty() {
            let opcode = state.read();
            if let Ok(opcode) = Opcode::try_from(opcode) {
                self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
            } else {
                self.instructions = vec![vec![]];
                log::warn!("invalid opcode {opcode:x}");
            }
        }

        for op in self.instructions.pop().expect("this can never be empty") {
            op(&mut state);
        }
        // if state.memState == Ready || Idle, read_pc
    }
}
