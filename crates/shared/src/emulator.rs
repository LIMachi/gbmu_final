use serde::{Deserialize, Serialize};

use super::breakpoints::Breakpoints;
use super::cpu::{self, Reg, Value};
use super::mem::{IOBus, MBCController};

pub trait Emulator: ReadAccess + Schedule {}

pub trait Bus: cpu::Bus + IOBus {}

pub trait BusWrapper {
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> Box<&dyn MBCController>;
}

impl<E: ReadAccess + Schedule> Emulator for E {}

impl<B: cpu::Bus + IOBus> Bus for B {}

pub trait Schedule {
    fn breakpoints(&mut self) -> &mut Breakpoints;
    fn play(&mut self);
    fn reset(&mut self);

    fn speed(&self) -> i32;
    fn speedup(&mut self);
    fn speeddown(&mut self);
}

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> Box<&dyn MBCController>;
}

pub trait State {
    type Storage: Serialize + for<'a> Deserialize<'a> + Sized;

    fn load_state(data: <Self as State>::Storage, ctx: &mut impl Emulator) -> Self;
    fn save_state(&self) -> <Self as State>::Storage;
}
