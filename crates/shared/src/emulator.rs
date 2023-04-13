use winit::event::VirtualKeyCode;
use super::breakpoints::Breakpoints;
use super::cpu::{self, Reg, Value};
use super::input::Section;
use super::mem::{IOBus, MBCController};

pub trait Emulator: ReadAccess + Schedule { }
pub trait Bus: cpu::Bus + IOBus { }

pub trait BusWrapper {
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> Box<&dyn MBCController>;
}

impl<E: ReadAccess + Schedule> Emulator for E { }
impl<B: cpu::Bus + IOBus> Bus for B { }

pub trait Schedule {
    fn breakpoints(&self) -> Breakpoints;
    fn play(&self);
    fn reset(&self);

    fn speed(&self) -> i32;
    fn set_speed(&self, speed: i32);
}

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> Box<&dyn MBCController>;
    fn binding(&self, key: VirtualKeyCode) -> Option<Section>;
}
