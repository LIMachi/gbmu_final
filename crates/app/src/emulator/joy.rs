use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use winit::event::{ElementState, KeyboardInput, ScanCode, VirtualKeyCode};
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, Mem};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Keys {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Right = 4,
    Left = 5,
    Up = 6,
    Down = 7,
}

pub struct Keybindings {
    bindings: HashMap<VirtualKeyCode, Keys>
}

impl Default for Keybindings {
    fn default() -> Self {
        //TODO read in a config file
        let mut bindings = HashMap::new();
        bindings.insert(VirtualKeyCode::A, Keys::Left);
        bindings.insert(VirtualKeyCode::D, Keys::Right);
        bindings.insert(VirtualKeyCode::S, Keys::Down);
        bindings.insert(VirtualKeyCode::W, Keys::Up);

        bindings.insert(VirtualKeyCode::I, Keys::A);
        bindings.insert(VirtualKeyCode::O, Keys::B);
        bindings.insert(VirtualKeyCode::K, Keys::Start);
        bindings.insert(VirtualKeyCode::L, Keys::Select);
        Self { bindings }
    }

}

#[derive(Default)]
pub struct Joypad {
    state: u8,
    joy: IOReg,
    int_flags: IOReg,
    bindings: Rc<RefCell<Keybindings>>
}

impl Joypad {
    pub fn new(bindings: Rc<RefCell<Keybindings>>) -> Self {
        Self { bindings, ..Default::default() }
    }

    pub fn handle(&mut self, event: KeyboardInput) {
        if let Some(keycode) = event.virtual_keycode {
            if let Some(&key) = self.bindings.as_ref().borrow().bindings.get(&keycode) {
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
