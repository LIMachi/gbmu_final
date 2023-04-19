use crate::io::IO;
use crate::mem::IOBus;

pub trait IODevice {
    fn write(&mut self, _io: IO, _v: u8, _bus: &mut dyn IOBus) {}
}

impl IODevice for () {}
