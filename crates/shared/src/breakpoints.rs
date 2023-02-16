use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use super::{Cpu, registers, value};

#[derive(Clone, Default)]
pub struct Breakpoints {
    breakpoints: Rc<RefCell<Vec<Breakpoint>>>
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Break {
    Cycles(usize),
    Instructions(usize),
    Register(registers::Reg, value::Value)
}

impl Break {
    pub fn tick(&mut self, runner: &impl Cpu) -> bool {
        match self {
            Break::Cycles(n) if *n == 0 => true,
            Break::Cycles(n) => { *n = *n - 1; false },
            Break::Instructions(n) if runner.done() && *n == 0 => true,
            Break::Instructions(n) if runner.done() => { *n = *n - 1; *n == 0 },
            Break::Register(r, v) if runner.register(*r) == *v => true,
            _ => false
        }
    }

    pub fn address(addr: u16) -> Self {
        Self::Register(registers::Reg::PC, addr.into())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Breakpoint {
    kind: Break,
    once: bool,
    pub enabled: bool
}

impl Breakpoint {
    fn new(kind: Break, once: bool) -> Self {
        Self { kind, once, enabled: true }
    }

    pub fn tick(&mut self, runner: &impl Cpu) -> (bool, bool) {
        (self.once, self.kind.tick(runner) && self.enabled)
    }

    pub fn pause() -> Self { Self::cycles(0) }
    pub fn step() -> Self { Self::instructions(1) }

    pub fn instructions(count: usize) -> Self {
        Self::new(Break::Instructions(count), true)
    }

    pub fn cycles(count: usize) -> Self {
        Self::new(Break::Cycles(count), true)
    }

    pub fn address(addr: u16) -> Self {
        Self::new(Break::address(addr), false)
    }

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

    pub fn display(&self) -> (registers::Reg, value::Value) {
        match self.kind {
            Break::Cycles(_) => unreachable!(),
            Break::Instructions(_) => unreachable!(),
            Break::Register(reg, value) => (reg, value)
        }
    }
}

impl Breakpoints {
    pub fn tick(&self, cpu: &impl Cpu) -> bool {
        let mut breakpoints = self.breakpoints.as_ref().borrow_mut();
        let mut stop = false;
        breakpoints.drain_filter(|bp| {
            let (once, res) = bp.tick(cpu);
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
