use crate::Bus;
use shared::{Target, cpu::{Reg, Value, Opcode}};
use super::{ops::*, State, Registers, decode::decode};

pub struct Cpu {
    instructions: Vec<Vec<Op>>,
    regs: Registers,
    cache: Vec<Value>,
    prefixed: bool,
    finished: bool
}

impl shared::Cpu for Cpu {
    fn done(&self) -> bool { self.finished }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Cpu {

    pub fn new(cgb: bool) -> Self {
        Self {
            instructions: Vec::new(),
            regs: if cgb { Registers::GBC } else { Registers::GB },
            cache: Vec::new(),
            prefixed: false,
            finished: false
        }
    }

    pub fn registers(&self) -> &Registers { &self.regs }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let prefixed = self.prefixed;
        self.prefixed = false;
        let mut state = State::new(bus, (&mut self.regs, &mut self.cache, &mut self.prefixed));
        if self.instructions.is_empty() {
            let opcode = state.read();
            if let Ok(opcode) = Opcode::try_from((opcode, prefixed)) {
                #[cfg(feature = "log_opcode")]
                log::debug!("[0x{:x}] instruction {opcode:?}", state.register(Reg::PC));
                self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
            } else {
                self.instructions = vec![vec![inc::pc]];
                log::warn!("invalid opcode {opcode:x}");
            }
        }
        for op in self.instructions.pop().expect("this can never be empty") {
            if op(&mut state) == BREAK {
                state.clear();
                self.instructions.clear();
                break;
            }
        }
        self.finished = self.instructions.is_empty() && !*state.prefix;
    }

    pub fn reset_finished(&mut self) { self.finished = false; }
}
