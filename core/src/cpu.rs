use crate::Bus;
use super::{ops::{Flow, Op}, Value, State, Registers, Opcode, CBOpcode};

pub struct Cpu {
    instructions: Vec<Vec<Op>>,
    regs: Registers,
    stack: Vec<Value>
}

impl Cpu {
    pub fn fetch(&mut self, state: &mut State) {
        let opcode = state.read();
        if let Ok(opcode) = Opcode::try_from(opcode) {
            self.instructions = match opcode {
                Opcode::Nop => vec![vec![]],
                Opcode::LdAA => vec![vec![]],
                _ => unimplemented!()
            }
        }
    }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        // TODO create state
        if let Some(ops) = self.instructions.pop() {
            // TODO run ops
        } else {

        }
        if self.instructions.is_empty() {

        }
    }
}

