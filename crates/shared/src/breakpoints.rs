use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use super::{Cpu, registers, value};
use serde::{Serialize, Deserialize};
use crate::cpu::{self, MemStatus, Opcode, Reg};
use crate::utils::Cell;

#[derive(Clone, Default)]
pub struct Breakpoints {
    breakpoints: Rc<RefCell<Vec<Breakpoint>>>
}

impl Breakpoints {
    pub fn new(breaks: Vec<Breakpoint>) -> Self {
        Self { breakpoints: breaks.cell() }
    }

    pub fn take(&self) -> Vec<Breakpoint> {
        self.breakpoints.as_ref().take()
    }
}

pub use super::io::Access;

//TODO add Read(u16) / Write(u16)
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Break {
    Access(Access, u16),
    Cycles(usize),
    Instructions(usize),
    Instruction(Opcode),
    Register(registers::Reg, value::Value)
}

impl PartialEq<MemStatus> for Break {
    fn eq(&self, other: &MemStatus) -> bool {
        match (self, other) {
            (Break::Access(Access::R | Access::RW, addr), MemStatus::ReqRead(n)) => addr == n,
            (Break::Access(Access::W | Access::RW, addr), MemStatus::ReqWrite(n)) => addr == n,
            _ => false
        }
    }
}

impl Break {
    pub fn tick(&mut self, runner: &impl Cpu, status: &MemStatus) -> bool {
        match self {
            Break::Cycles(n) if *n == 0 => true,
            Break::Cycles(n) => { *n = *n - 1; false },
            Break::Instruction(op) if runner.done() && runner.previous() == *op => true,
            Break::Instructions(n) if runner.done() && *n == 0 => true,
            Break::Instructions(n) if runner.done() => { *n = *n - 1; *n == 0 },
            Break::Register(r, v) if runner.done() && runner.register(*r) == *v => true,
            acc @Break::Access( .. ) if acc == status => true,
            _ => false
        }
    }

    pub fn address(addr: u16) -> Self {
        Self::Register(registers::Reg::PC, addr.into())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Breakpoint {
    kind: Break,
    once: bool,
    pub enabled: bool
}

impl Breakpoint {
    fn new(kind: Break, once: bool) -> Self {
        Self { kind, once, enabled: true }
    }
}

impl Breakpoint {
    pub fn tick(&mut self, runner: &impl Cpu, status: &MemStatus) -> (bool, bool) {
        (self.once, self.kind.tick(runner, status) && self.enabled)
    }

    pub fn pause() -> Self { Self::cycles(0) }
    pub fn step() -> Self { Self::instructions(1) }

    pub fn instructions(count: usize) -> Self {
        Self::new(Break::Instructions(count), true)
    }

    pub fn instruction(ins: Opcode) -> Self { Self::new(Break::Instruction(ins), false) }

    pub fn cycles(count: usize) -> Self {
        Self::new(Break::Cycles(count), true)
    }

    pub fn address(addr: u16) -> Self {
        Self::new(Break::address(addr), false)
    }

    pub fn access(addr: u16, access: Access) -> Self { Self::new(Break::Access(access, addr), false) }

    pub fn register(reg: registers::Reg, value: value::Value) -> Self {
        Self::new(Break::Register(reg, value), false)
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }

    pub fn temp(&self) -> bool { self.once }

    pub fn display(&self) -> String {
        match self.kind {
            Break::Cycles(_) => unreachable!(),
            Break::Instructions(_) => unreachable!(),
            Break::Register(reg, value) => format!("{reg:?} == {value:#06x}"),
            Break::Instruction(op) => crate::opcodes::dbg::dbg_opcodes(op).1.to_string(),
            Break::Access(access, addr) => format!("{addr:#06X}:{}", access.format()),
        }
    }
}

impl Breakpoints {
    pub fn tick(&self, cpu: &impl Cpu, status: cpu::MemStatus) -> bool {
        let mut breakpoints = self.breakpoints.as_ref().borrow_mut();
        let mut stop = false;
        breakpoints.drain_filter(|bp| {
            let (once, res) = bp.tick(cpu, &status);
            stop |= res;
            once && res
        });
        !stop
    }

    pub fn bp_mut(&self) -> RefMut<Vec<Breakpoint>> {
        self.breakpoints.as_ref().borrow_mut()
    }

    pub fn pause(&self) {
        self.breakpoints.as_ref().borrow_mut().push(Breakpoint::pause());
    }
    pub fn step(&self) {
        self.breakpoints.as_ref().borrow_mut().push(Breakpoint::step());
    }

    pub fn schedule(&self, bp: Breakpoint) {
        self.breakpoints.as_ref().borrow_mut().push(bp);
    }
}
