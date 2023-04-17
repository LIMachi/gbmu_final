use std::fmt::Formatter;
use serde::{Serialize, Deserialize};
use crate::{
    value,
    cpu::{Cpu, Op, Opcode, Reg},
    utils::{convert::Converter}
};

#[derive(Default)]
pub struct Breakpoints {
    breakpoints: Vec<Breakpoint>
}

impl Breakpoints {
    pub fn new(breaks: Vec<Breakpoint>) -> Self {
        Self { breakpoints: breaks }
    }

    pub fn take(&mut self) -> Vec<Breakpoint> {
        std::mem::take(&mut self.breakpoints)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Any,
    Eq(u8),
    And(u8),
    Not(u8)
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Any, Any) => true,
            (Eq(_), Eq(_)) => true,
            (And(_), And(_)) => true,
            (Not(_), Not(_)) => true,
            _ => false
        }
    }
}

impl Default for Value {
    fn default() -> Self { Self::Any }
}

impl PartialEq<u8> for Value {
    fn eq(&self, other: &u8) -> bool {
        match (self, *other) {
            (Value::Any, _) => true,
            (Value::Eq(u), v) if u == &v => true,
            (Value::And(u), v) if u & v == *u => true,
            (Value::Not(u), v) if u != &v => true,
            _ => false
        }
    }
}

impl Value {
    pub fn param(&self) -> bool { self != &Value::Any }
    pub fn sym(&self) -> &str {
        match self {
            Value::Any => "*",
            Value::Eq(_) => "==",
            Value::Not(_) => "!=",
            Value::And(_) => "&",
        }
    }

    pub fn parse(&mut self, input: &mut String) {
        match self {
            Value::Eq(v) | Value::And(v) | Value::Not(v) => *v = u8::convert(&input),
            _ => {}
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Any => f.write_fmt(format_args!("= *")),
            Value::Not(v) => f.write_fmt(format_args!("!= {v:#02x}")),
            Value::Eq(v) => f.write_fmt(format_args!("== {v:#02x}")),
            Value::And(v) => f.write_fmt(format_args!("& {v:#02x}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Access {
    addr: u16,
    kind: super::io::Access,
    value: Value
}

impl Access {
    fn new(addr: u16, kind: super::io::Access, value: Value) -> Self {
        Self { addr, kind, value }
    }

    pub fn read(addr: u16, value: Value) -> Self { Access::new(addr, super::io::Access::R, value) }
    pub fn write(addr: u16, value: Value) -> Self { Access::new(addr, super::io::Access::W, value) }
    pub fn rw(addr: u16, value: Value) -> Self { Access::new(addr, super::io::Access::RW, value) }

    pub fn matches(&self, op: Op) -> bool {
        use super::io::Access as Kind;
        match (self.kind, op) {
            (Kind::R, Op::Read(addr, v)) |
            (Kind::RW | Kind::W, Op::Write(addr, v))
            if addr == self.addr => self.value == v,
            _ => false
        }
    }

    pub fn format(&self) -> String {
        format!("{:#06X}{:?} {}", self.addr, self.kind, self.value)
    }
}

//TODO add Read(u16) / Write(u16)
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum Break {
    Access(Access),
    Cycles(usize),
    Instructions(usize),
    Instruction(Opcode),
    Register(Reg, value::Value)
}

impl Break {
    pub fn tick(&mut self, runner: &impl Cpu, last: Option<Op>) -> bool {
        match self {
            Break::Cycles(n) if *n == 0 => true,
            Break::Cycles(n) => { *n = *n - 1; false },
            Break::Instruction(op) if runner.done() && runner.previous() == *op => true,
            Break::Instructions(n) if runner.done() && *n == 0 => true,
            Break::Instructions(n) if runner.done() => { *n = *n - 1; *n == 0 },
            Break::Register(r, v) if runner.done() && runner.register(*r) == *v => true,
            Break::Access(access) if let Some(last) = last => access.matches(last),
            _ => false
        }
    }

    pub fn address(addr: u16) -> Self {
        Self::Register(Reg::PC, addr.into())
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
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
    pub fn tick(&mut self, runner: &impl Cpu, last: Option<Op>) -> (bool, bool) {
        (self.once, self.kind.tick(runner, last) && self.enabled)
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

    pub fn access(access: Access) -> Self { Self::new(Break::Access(access), false) }

    pub fn register(reg: Reg, value: value::Value) -> Self {
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
            Break::Access(access) => access.format(),
        }
    }
}

impl Breakpoints {
    pub fn tick(&mut self, cpu: &impl Cpu, last: Option<Op>) -> bool {
        let mut stop = false;
        self.breakpoints.drain_filter(|bp| {
            let (once, res) = bp.tick(cpu, last);
            log::info!("{bp:?} {res}");
            stop |= res;
            once && res
        });
        !stop
    }

    pub fn bp_mut(&mut self) -> &mut Vec<Breakpoint> { &mut self.breakpoints }

    pub fn pause(&mut self) {
        self.breakpoints.push(Breakpoint::pause());
    }
    pub fn step(&mut self) {
        self.breakpoints.push(Breakpoint::step());
    }

    pub fn schedule(&mut self, bp: Breakpoint) {
        self.breakpoints.push(bp);
    }
}
