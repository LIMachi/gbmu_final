use crate::Bus;
use shared::{cpu::{Reg, Value, Opcode}};
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
    ime: bool
}

impl shared::cpu::Cpu for Cpu {
    fn done(&self) -> bool { self.finished }
    fn previous(&self) -> Opcode { self.prev }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            mode: Mode::Running,
            prev: Opcode::Nop,
            instructions: Vec::new(),
            regs: Registers::default(),
            cache: Vec::new(),
            prefixed: false,
            finished: false,
            ime: false,
            at: 0,
        }
    }
}

impl Cpu {
    pub fn skip_boot(&mut self, cgb: bool) {
        self.regs = if cgb { Registers::GBC } else { Registers::GB };
    }

    pub fn registers(&self) -> &Registers { &self.regs }

    fn check_interrupts(&mut self, bus: &mut dyn Bus) {
        if !self.instructions.is_empty() || self.prev == Opcode::Ei { return };
        let int = bus.interrupt();
        if int != 0 {
            if self.mode == Mode::Halt { self.mode = Mode::Running };
            if self.ime {
                self.ime = false;
                let (bit, ins) = super::decode::interrupt(int);
                bus.int_reset(bit);
                self.instructions = ins.iter().rev().map(|x| x.to_vec()).collect();
            }
        }
    }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let prefixed = self.prefixed;
        self.prefixed = false;
        if !prefixed { self.check_interrupts(bus); }
        if self.mode == Mode::Halt {
            return ;
        }
        let mut state = State::new(bus, (&mut self.regs, &mut self.cache, &mut self.prefixed, &mut self.ime, &mut self.mode));
        if self.instructions.is_empty() {
            let opcode = state.read();
            let Ok(opcode) = Opcode::try_from((opcode, prefixed)) else { unreachable!(); };

            #[cfg(feature = "log_opcode")]
            {
                use std::io::Write;
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
                    let buff = format!("\
                    A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X}\
                    SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X} INS: {:?} NEXT: {:?}\n",
                                       a, f, b, c, d, e, h, l, sp, pc, pc0, pc1, pc2, pc3, self.prev, opcode);
                    log::info!("{buff}");
                    file.write_all(buff.as_bytes());
                }
            }

            self.instructions = decode(opcode).iter().rev().map(|x| x.to_vec()).collect();
            self.prev = opcode;
            self.at = state.register(Reg::PC).u16();
            if let Opcode::Invalid(n) = opcode {
                log::warn!("invalid opcode {n:#02x}");
            }
        }
        for op in self.instructions.pop().expect("this can never be empty") {
            if op(&mut state) == BREAK {
                state.clear();
                self.instructions.clear();
                break;
            }
        }
        self.finished = self.instructions.is_empty();
    }

    pub fn reset_finished(&mut self) { self.finished = false; }
}
