use shared::events::*;
use shared::io::{IO, IORegs};
use shared::mem::{Device, IOBus};

use shared::input::{Section, Keybindings};

#[derive(Default)]
pub struct Joypad {
    state: u8,
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

    pub fn tick(&mut self, io: &mut IORegs) {
        let joy = io.io(IO::JOYP);
        let p4 = joy.bit(4);
        let p5 = joy.bit(5);
        let dir = if p4 == 0 { self.state >> 4 } else { 0 };
        let act = if p5 == 0 { self.state & 0xF } else { 0 };
        let p = joy.value() & 0xF;
        let v =  0xF ^ (dir | act);
        let int = (p ^ v) & p != 0;
        joy.direct_write((p4 << 4) | (p5 << 5) | v);
        if int { { io.io(IO::IF).set(4); } }
    }
}

impl Device for Joypad {}
