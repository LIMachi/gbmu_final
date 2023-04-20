use shared::input::KeyCat;
use shared::io::{IO, IODevice, IORegs};
use shared::mem::IOBus;

#[derive(Default)]
pub struct Joypad {
    state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn update(&mut self, io: &mut IORegs) {
        let joy = io.io_mut(IO::JOYP);
        let p = joy.value() & 0xF;
        let mut v = 0;
        let p4 = joy.bit(4);
        let p5 = joy.bit(5);
        if p4 != 0 { v |= self.state >> 4; }
        if p5 != 0 { v |= self.state & 0xF; }
        let int = (p ^ v) & p != 0;
        joy.direct_write(v | (p4 << 4) | (p5 << 5));
        if int { io.int_set(4); }
    }
}

impl shared::input::Joypad for Joypad {
    fn update(&mut self, key: KeyCat, pressed: bool, io: &mut IORegs) {
        if let KeyCat::Joy(key) = key {
            let mask = 1 << (key as u8);
            self.state = (self.state & !mask) | if pressed { mask } else { 0 };
            self.update(io);
        }
    }
}

impl IODevice for Joypad {
    fn write(&mut self, io: IO, _: u8, bus: &mut dyn IOBus) {
        if io == IO::JOYP { self.update(bus.io_regs()) }
    }
}
