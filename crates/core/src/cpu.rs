use crate::Bus;
use shared::{Target, cpu::{Reg, Value, Opcode}};
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use super::{ops::*, State, Registers, decode::decode};

pub struct Cpu {
    prev: Opcode,
    instructions: Vec<Vec<Op>>,
    regs: Registers,
    cache: Vec<Value>,
    prefixed: bool,
    finished: bool,
    ime: bool,
    int_flags: IOReg,
    ie: IOReg
}

impl shared::Cpu for Cpu {
    fn done(&self) -> bool { self.finished }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Cpu {

    pub fn new(cgb: bool) -> Self {
        Self {
            prev: Opcode::Nop,
            instructions: Vec::new(),
            regs: if cgb { Registers::GBC } else { Registers::GB },
            cache: Vec::new(),
            prefixed: false,
            finished: false,
            ime: false,
            int_flags: IOReg::unset(),
            ie: IOReg::unset(),
        }
    }

    pub fn registers(&self) -> &Registers { &self.regs }

    fn check_interrupts(&mut self) {
        if !self.instructions.is_empty() || self.prev == Opcode::Ei { return };
        let int = (self.int_flags.read() & self.ie.read()) & 0x1F;
        if self.ime && int != 0 {
            self.ime = false;
            let (bit, ins) = super::decode::interrupt(int);
            self.int_flags.reset(bit);
            self.instructions = ins.iter().rev().map(|x| x.to_vec()).collect();
        }
    }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let prefixed = self.prefixed;
        self.prefixed = false;
        self.check_interrupts();
        let mut state = State::new(bus, (&mut self.regs, &mut self.cache, &mut self.prefixed, &mut self.ime));
        if self.instructions.is_empty() {
            let opcode = state.read();
            if let Ok(opcode) = Opcode::try_from((opcode, prefixed)) {
                #[cfg(feature = "log_opcode")]
                log::debug!("[0x{:x}] instruction {opcode:?}", state.register(Reg::PC));
                self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
                self.prev = opcode;
            } else {
                self.instructions = vec![vec![inc::pc]];
                self.prev = Opcode::Nop;
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

impl Device for Cpu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ie = bus.io(IO::IE);
        self.int_flags = bus.io(IO::IF);
    }
}
