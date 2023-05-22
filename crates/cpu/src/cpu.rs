use serde::{Deserializer, Serializer};
use shared::{cpu::{Opcode, Reg, Value}};
use shared::serde::{Deserialize, Serialize};

use crate::Bus;

use super::{decode::decode, ops::*, Registers, State};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Mode {
    Running,
    Halt,
    Stop,
}

pub struct Cpu {
    prev: Opcode,
    at: u16,
    mode: Mode,
    instructions: &'static [&'static [Op]],
    ins: usize,
    count: usize,
    regs: Registers,
    cache: Vec<Value>,
    prefixed: bool,
    finished: bool,
    ime: bool,
    doctor: Option<std::fs::File>, //TODO serde: rebind
    stop: usize,
}

impl Clone for Cpu {
    fn clone(&self) -> Self {
        Self {
            prev: self.prev,
            at: self.at,
            mode: self.mode,
            instructions: self.instructions,
            ins: self.ins,
            count: self.count,
            regs: self.regs.clone(),
            cache: self.cache.clone(),
            prefixed: self.prefixed,
            finished: self.finished,
            ime: self.ime,
            doctor: None,
            stop: self.stop
        }
    }
}

#[derive(Serialize, Deserialize)]
struct InnerCpu {
    prev: Opcode,
    at: u16,
    mode: Mode,
    ins: usize,
    count: usize,
    regs: Registers,
    cache: Vec<Value>,
    prefixed: bool,
    finished: bool,
    ime: bool,
    stop: usize,
}

impl Serialize for Cpu {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        InnerCpu{prev: self.prev, at: self.at, mode: self.mode, ins: self.ins, count: self.count, regs: self.regs, cache: self.cache.clone(), prefixed: self.prefixed, finished: self.finished, ime: self.ime, stop: self.stop}.serialize(serializer)
    }
}

impl <'de> Deserialize<'de> for Cpu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Deserialize::deserialize(deserializer).map(|inner: InnerCpu| {
            Cpu{
                prev: inner.prev,
                at: inner.at,
                mode: inner.mode,
                instructions: decode(inner.prev),
                ins: inner.ins,
                count: inner.count,
                regs: inner.regs,
                cache: inner.cache,
                prefixed: inner.prefixed,
                finished: inner.finished,
                ime: inner.ime,
                doctor: None,
                stop: inner.stop
            }
        })

    }
}

impl shared::cpu::Cpu for Cpu {
    fn done(&self) -> bool { self.finished }
    fn previous(&self) -> Opcode { self.prev }
    fn register(&self, reg: Reg) -> Value { self.regs.read(reg) }
}

impl Default for Cpu {
    fn default() -> Self {
        #[cfg(feature = "log_opcode")]
            let doctor = std::fs::File::open("opcodes.log").ok();
        #[cfg(not(feature = "log_opcode"))]
            let doctor = None;
        Self {
            mode: Mode::Running,
            prev: Opcode::Nop,
            instructions: decode(Opcode::Nop),
            ins: 1,
            count: 1,
            regs: Registers::default(),
            cache: Vec::new(),
            prefixed: false,
            finished: false,
            ime: false,
            at: 0,
            doctor,
            stop: 0,
        }
    }
}

impl Cpu {

    pub fn reload(self, load: Self) -> Self {
        let mut t = load;
        t.doctor = self.doctor;
        t
    }

    pub fn skip_boot(&mut self, cgb: bool) {
        self.regs = if cgb { Registers::GBC } else { Registers::GB };
    }

    pub fn registers(&self) -> &Registers { &self.regs }

    fn check_interrupts(&mut self, bus: &mut dyn Bus) {
        if self.ins < self.count || self.prev == Opcode::Ei { return; };
        let int = bus.interrupt();
        if int != 0 {
            if self.mode == Mode::Halt { self.mode = Mode::Running };
            if self.ime {
                self.ime = false;
                let (bit, ins) = super::decode::interrupt(int);
                bus.int_reset(bit);
                self.instructions = ins;
                self.count = self.instructions.len();
                self.ins = 0;
            }
        }
    }

    pub fn stopped(&self) -> bool { self.mode == Mode::Stop }

    pub fn cycle(&mut self, bus: &mut dyn Bus) {
        let prefixed = self.prefixed;
        self.prefixed = false;
        if !prefixed { self.check_interrupts(bus); }
        if self.mode == Mode::Halt {
            return;
        }
        if self.mode == Mode::Stop {
            if self.stop == 0 {
                return;
            } else {
                self.stop -= 1;
                if self.stop == 0 {
                    self.mode = Mode::Running;
                    bus.toggle_ds();
                }
            }
        }
        let mut state = State::new(bus, (
            &mut self.regs, &mut self.cache, &mut self.prefixed, &mut self.ime, &mut self.mode, &mut self.stop),
        );
        if self.ins >= self.count {
            let opcode = state.read();
            let Ok(opcode) = Opcode::try_from((opcode, prefixed)) else { unreachable!(); };

            self.doctor.as_mut().map(|x| {
                use std::io::Write;
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
                    x.write_all(buff.as_bytes()).ok();
                }
            });
            self.instructions = decode(opcode);
            self.ins = 0;
            self.count = self.instructions.len();
            self.prev = opcode;
            self.at = state.register(Reg::PC).u16();
            if let Opcode::Invalid(n) = opcode {
                log::warn!("invalid opcode {n:#02x}");
            }
        }
        for op in self.instructions[self.ins] {
            if op(&mut state) == BREAK {
                state.clear();
                self.ins = self.count;
                break;
            }
        }
        self.ins += 1;
        self.finished = self.ins >= self.count;
    }

    pub fn reset_finished(&mut self) { self.finished = false; }
}
