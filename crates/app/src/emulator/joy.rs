use winit::event::{ElementState, KeyboardInput};
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

use shared::input::{Section, Keybindings};

#[derive(Default)]
pub struct Joypad {
    state: u8,
    joy: IOReg,
    int_flags: IOReg,
    bindings: Keybindings
}

impl Joypad {
    pub fn new(bindings: Keybindings) -> Self {
        Self { bindings, ..Default::default() }
    }

    pub fn handle(&mut self, event: KeyboardInput) {
        if let Some(keycode) = event.virtual_keycode {
            if let Some(Section::Joy(key)) = self.bindings.get(keycode) {
                let mask = 1u8 << (key as u8);
                self.state = (self.state & !mask) | if event.state == ElementState::Pressed { mask } else { 0 };
            }
        }
    }

    pub fn tick(&mut self) {
        let p4 = self.joy.bit(4);
        let p5 = self.joy.bit(5);
        let dir = if p4 == 0 { self.state >> 4 } else { 0 };
        let act = if p5 == 0 { self.state & 0xF } else { 0 };
        let p = self.joy.value() & 0xF;
        let v =  0xF ^ (dir | act);
        if (p ^ v) & p != 0 { self.int_flags.set(4); }
        self.joy.direct_write((p4 << 4) | (p5 << 5) | v);
    }
}

impl Device for Joypad {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.joy = bus.io(IO::JOYP);
        self.int_flags = bus.io(IO::IF);
    }
}
