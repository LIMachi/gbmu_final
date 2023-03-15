use std::io::Write;
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
    cgb: IOReg,
    int_flags: IOReg,
    ie: IOReg
}

impl shared::Cpu for Cpu {
    fn done(&self) -> bool { self.finished }
    fn previous(&self) -> Opcode { self.prev }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Cpu {

    pub fn new() -> Self {
        Self {
            mode: Mode::Running,
            prev: Opcode::Nop,
            instructions: Vec::new(),
            regs: Registers::default(),
            cache: Vec::new(),
            prefixed: false,
            finished: false,
            ime: false,
            cgb: IOReg::unset(),
            int_flags: IOReg::unset(),
            ie: IOReg::unset(),
            at: 0,
        }
    }

    pub fn skip_boot(&mut self) {
        self.regs = if self.cgb.read() != 0 { Registers::GBC } else { Registers::GB }
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
            #[cfg(feature = "log_opcode")]
            {
                static mut OUT: Option<std::fs::File> = None;
                let file = unsafe { OUT.as_mut().unwrap_or_else(|| {
                    OUT = Some(std::fs::File::create("out.log").unwrap()); OUT.as_mut().unwrap()
                } ) };
                if !prefixed {
                    let (a, f, b, c, d, e, h, l, sp, pc) = (
                        state.register(Reg::A).u8(), state.register(Reg::F).u8(), state.register(Reg::B).u8(), state.register(Reg::C).u8(),
                        state.register(Reg::D).u8(), state.register(Reg::E).u8(), state.register(Reg::H).u8(), state.register(Reg::L).u8(),
                        state.register(Reg::SP).u16(), state.register(Reg::PC).u16());
                    let [pc0, pc1, pc2, pc3] = [
                        state.bus.direct_read(pc),
                        state.bus.direct_read(pc + 1),
                        state.bus.direct_read(pc + 2),
                        state.bus.direct_read(pc + 3)];
                    file.write_all(format!("A:{:02X} F:{:02X} SP:{:04X} PC:{:04X} INS:{:?}\n", a, f, sp, pc, self.prev).as_bytes());
                }
            }
            let opcode = state.read();
            let Ok(opcode) = Opcode::try_from((opcode, prefixed)) else { unreachable!(); };
            self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
            self.prev = opcode;
            self.at = state.register(Reg::PC).u16();
            if let Opcode::Invalid(n) = opcode {
                log::warn!("invalid opcode {n:#02x}");
            }
        }
        let mut ok = true;
        for op in self.instructions.pop().expect("this can never be empty") {
            if op(&mut state) == BREAK {
                state.clear();
                self.instructions.clear();
                ok = false;
                break;
            }
        }
        self.finished = self.instructions.is_empty();
    }

    pub fn reset_finished(&mut self) { self.finished = false; }
}

impl Device for Cpu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.cgb = bus.io(IO::CGB);
        self.ie = bus.io(IO::IE);
        self.int_flags = bus.io(IO::IF);
    }
}
