use crate::io::IO;
use crate::mem::IOBus;

pub trait IODevice {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {}
}

impl IODevice for () {}
