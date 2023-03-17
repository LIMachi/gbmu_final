use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Display;
use std::rc::Rc;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Shortcut {
    Pause,
    Run,
    Step,
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Section {
    Joy(Keys),
    Dbg(Shortcut)
}

impl Section {
    pub fn joypad() -> impl IntoIterator<Item = Section> {
        [Section::Joy(Keys::Left),
        Section::Joy(Keys::Right),
        Section::Joy(Keys::Down),
        Section::Joy(Keys::Up),
        Section::Joy(Keys::A),
        Section::Joy(Keys::B),
        Section::Joy(Keys::Start),
        Section::Joy(Keys::Select)]
    }

    pub fn shortcuts() -> impl IntoIterator<Item = Section> {
        [Section::Dbg(Shortcut::Pause),
        Section::Dbg(Shortcut::Run),
        Section::Dbg(Shortcut::Step),]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Keybindings {
    bindings: HashMap<VirtualKeyCode, Section>,
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(VirtualKeyCode::A, Section::Joy(Keys::Left));
        bindings.insert(VirtualKeyCode::D, Section::Joy(Keys::Right));
        bindings.insert(VirtualKeyCode::S, Section::Joy(Keys::Down));
        bindings.insert(VirtualKeyCode::W, Section::Joy(Keys::Up));

        bindings.insert(VirtualKeyCode::Space, Section::Joy(Keys::A));
        bindings.insert(VirtualKeyCode::LShift, Section::Joy(Keys::B));
        bindings.insert(VirtualKeyCode::Z, Section::Joy(Keys::Start));
        bindings.insert(VirtualKeyCode::X, Section::Joy(Keys::Select));

        bindings.insert(VirtualKeyCode::F2, Section::Dbg(Shortcut::Pause));
        bindings.insert(VirtualKeyCode::F9, Section::Dbg(Shortcut::Run));
        bindings.insert(VirtualKeyCode::F3, Section::Dbg(Shortcut::Step));
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
            if let Some(Section::Joy(key)) = self.bindings.as_ref().borrow().bindings.get(&keycode) {
                let mask = 1u8 << (*key as u8);
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

impl Keybindings {
    pub fn set(&mut self, key: Section, code: VirtualKeyCode) {
        self.bindings.drain_filter(|_v , x| &key == x);
        self.bindings.insert(code, key);
    }

    pub fn get(&self, key: Section) -> (Section, Option<VirtualKeyCode>) {
        (key, self.bindings.iter().find(|(_v, s)| s == &&key ).map(|(v, _)| *v ))
    }
}
