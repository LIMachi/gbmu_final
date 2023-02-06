use log::Level::Debug;
use crate::{Bus, Reg};
use super::{ops::*, Value, State, Registers, Opcode, CBOpcode, decode::decode};

pub struct Cpu {
    instructions: Vec<Vec<Op>>,
    regs: Registers,
    cache: Vec<Value>,
    just_finished: bool
}

impl Cpu {

    pub fn new(target: super::Target) -> Self {
        Self {
            instructions: Vec::new(),
            regs: match target { super::Target::GB => Registers::GB, super::Target::GBC => Registers::GBC },
            cache: Vec::new(),
            just_finished: false
        }
    }

    pub fn registers(&self) -> &Registers {
        &self.regs
    }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let mut state = State::new(bus, &mut self.regs, &mut self.cache);
        if self.instructions.is_empty() {
            self.just_finished = false;
            let opcode = state.read();
            if let Ok(opcode) = Opcode::try_from(opcode) {
                #[cfg(feature = "log_opcode")]
                log::debug!("[0x{:x}] instruction {opcode:?}", state.register(Reg::PC));
                self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
            } else {
                self.instructions = vec![vec![inc::pc]];
                log::warn!("invalid opcode {opcode:x}");
            }
        }
        for op in self.instructions.pop().expect("this can never be empty") {
            if op(&mut state) == BREAK { // assuming pc is already set to next instruction, else kaboom
                state.clear();
                self.instructions.clear();
                break;
            }
        }
        if self.instructions.is_empty() { self.just_finished = true }
        // State drop will update bus
    }
}
