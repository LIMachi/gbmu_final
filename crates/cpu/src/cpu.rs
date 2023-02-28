use crate::Bus;
use shared::{Target, cpu::{Reg, Value, Opcode}};
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use super::{ops::*, State, Registers, decode::decode};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    Running,
    Halt,
}

pub struct Cpu {
    prev: Opcode,
    at: u16,
    mode: Mode,
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
    fn previous(&self) -> Opcode { self.prev }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Cpu {

    pub fn new(cgb: bool) -> Self {
        Self {
            mode: Mode::Running,
            prev: Opcode::Nop,
            instructions: Vec::new(),
            regs: if cgb { Registers::GBC } else { Registers::GB },
            cache: Vec::new(),
            prefixed: false,
            finished: false,
            ime: false,
            int_flags: IOReg::unset(),
            ie: IOReg::unset(),
            at: 0,
        }
    }

    pub fn registers(&self) -> &Registers { &self.regs }

    fn check_interrupts(&mut self) {
        if !self.instructions.is_empty() || self.prev == Opcode::Ei { return };
        let int = (self.int_flags.read() & self.ie.read()) & 0x1F;
        if int != 0 {
            if self.mode == Mode::Halt { self.mode = Mode::Running };
            if self.ime {
                self.ime = false;
                let (bit, ins) = super::decode::interrupt(int);
                self.int_flags.reset(bit);
                self.instructions = ins.iter().rev().map(|x| x.to_vec()).collect();
            }
        }
    }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let prefixed = self.prefixed;
        self.prefixed = false;
        self.check_interrupts();
        if self.mode == Mode::Halt {
            return ;
        }
        let mut state = State::new(bus, (&mut self.regs, &mut self.cache, &mut self.prefixed, &mut self.ime, &mut self.mode));
        if self.instructions.is_empty() {
            let opcode = state.read();
            if let Ok(opcode) = Opcode::try_from((opcode, prefixed)) {
                self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
                self.prev = opcode;
                self.at = state.register(Reg::PC).u16();
            } else {
                self.instructions = vec![vec![inc::pc]];
                self.prev = Opcode::Nop;
                log::warn!("invalid opcode {opcode:x}");
            }
        }
        let mut ok = true;
        for op in self.instructions.pop().expect("this can never be empty") {
            if op(&mut state) == BREAK {
                #[cfg(feature = "log_opcode")]
                log::debug!("[0x{:x}] instruction {:?} [BREAK]", self.at, self.prev);
                state.clear();
                self.instructions.clear();
                ok = false;
                break;
            }
        }
        if ok && self.instructions.is_empty() && [
            Opcode::Ret, Opcode::Reti, Opcode::RetNZ, Opcode::RetNC, Opcode::RetC, Opcode::RetZ,
            Opcode::Calla16, Opcode::CallNZa16, Opcode::CallZa16, Opcode::CallNCa16,Opcode::CallCa16,
            Opcode::Rst00H, Opcode::Rst08H, Opcode::Rst10H, Opcode::Rst18H, Opcode::Rst28H, Opcode::Rst20H, Opcode::Rst30H, Opcode::Rst38H,
            Opcode::Jpa16, Opcode::Jrr8, Opcode::JpHL, Opcode::JrCr8, Opcode::JrZr8, Opcode::JrNCr8, Opcode::JrNZr8,
            Opcode::JpCa16, Opcode::JpZa16, Opcode::JpNZa16, Opcode::JpNCa16, Opcode::Ei, Opcode::Di, Opcode::LdhInda8A, Opcode::LdhAInda8
        ].contains(&self.prev) {
            #[cfg(feature = "log_opcode")]
            log::debug!("[0x{:x}] instruction {:?}", self.at, self.prev);
        }
        self.finished = self.instructions.is_empty();
    }

    pub fn reset_finished(&mut self) { self.finished = false; }
}

impl Device for Cpu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ie = bus.io(IO::IE);
        self.int_flags = bus.io(IO::IF);
    }
}
