use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use winit::event::{ButtonId, DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoopProxy;

use crate::Events;
use crate::io::IORegs;

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Debug {
    Pause,
    Run,
    Step,
    Reset,
}

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Shortcut {
    SpeedUp,
    SpeedDown,
    Save,
    Quit,
    SaveState,
    LoadState,
}

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyCat {
    Joy(Keys),
    Dbg(Debug),
    Game(Shortcut),
}

#[derive(Hash, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Input {
    Keyboard(VirtualKeyCode, ModifiersState),
    Device(ButtonId),
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Keyboard(key, state) => {
                let prefix = if state.ctrl() { "Ctrl + " } else if state.shift() { "Shift + " } else if state.alt() { "Alt + " } else { "" };
                f.write_fmt(format_args!("{prefix}{key:?}"))
            }
            Input::Device(button) => f.write_fmt(format_args!("Button{button}"))
        }
    }
}

impl Input {
    pub fn key(code: VirtualKeyCode) -> Self {
        Self::Keyboard(code, ModifiersState::empty())
    }

    pub fn pressed(&self, event: &Event<Events>, modifiers: &ModifiersState) -> bool {
        match (self, event) {
            (Input::Keyboard(code, state), Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(key), state: pressed, ..
                    }, ..
                }, ..
            }) if pressed == &ElementState::Pressed && code == key => {
                let mut mods = *modifiers;
                match key {
                    VirtualKeyCode::LShift | VirtualKeyCode::RShift => mods.remove(ModifiersState::SHIFT),
                    VirtualKeyCode::LControl | VirtualKeyCode::RControl => mods.remove(ModifiersState::CTRL),
                    VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => mods.remove(ModifiersState::ALT),
                    _ => {}
                };
                state == &mods
            }
            (Input::Device(key), Event::DeviceEvent {
                event: DeviceEvent::Button {
                    button, state
                }, ..
            }) if state == &ElementState::Pressed && key == button => true,
            _ => false
        }
    }
}

impl KeyCat {
    pub fn joypad() -> impl IntoIterator<Item=KeyCat> {
        [KeyCat::Joy(Keys::Left),
            KeyCat::Joy(Keys::Right),
            KeyCat::Joy(Keys::Down),
            KeyCat::Joy(Keys::Up),
            KeyCat::Joy(Keys::A),
            KeyCat::Joy(Keys::B),
            KeyCat::Joy(Keys::Start),
            KeyCat::Joy(Keys::Select)
        ]
    }

    pub const fn debug() -> impl IntoIterator<Item=KeyCat> {
        [KeyCat::Dbg(Debug::Pause),
            KeyCat::Dbg(Debug::Run),
            KeyCat::Dbg(Debug::Step),
            KeyCat::Dbg(Debug::Reset),
        ]
    }

    pub const fn shortcuts() -> impl IntoIterator<Item=KeyCat> {
        [KeyCat::Game(Shortcut::SpeedDown),
            KeyCat::Game(Shortcut::SpeedUp),
            KeyCat::Game(Shortcut::Save),
            KeyCat::Game(Shortcut::Quit),
            KeyCat::Game(Shortcut::SaveState),
            KeyCat::Game(Shortcut::LoadState),
        ]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Keybindings {
    bindings: HashMap<Input, KeyCat>,
    #[serde(skip)]
    inputs: HashMap<KeyCat, (Input, ElementState)>,
    #[serde(skip)]
    modifiers: ModifiersState,
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Input::key(VirtualKeyCode::A), KeyCat::Joy(Keys::Left));
        bindings.insert(Input::key(VirtualKeyCode::D), KeyCat::Joy(Keys::Right));
        bindings.insert(Input::key(VirtualKeyCode::S), KeyCat::Joy(Keys::Down));
        bindings.insert(Input::key(VirtualKeyCode::W), KeyCat::Joy(Keys::Up));

        bindings.insert(Input::key(VirtualKeyCode::Space), KeyCat::Joy(Keys::A));
        bindings.insert(Input::key(VirtualKeyCode::LShift), KeyCat::Joy(Keys::B));
        bindings.insert(Input::key(VirtualKeyCode::Z), KeyCat::Joy(Keys::Start));
        bindings.insert(Input::key(VirtualKeyCode::X), KeyCat::Joy(Keys::Select));

        bindings.insert(Input::key(VirtualKeyCode::F2), KeyCat::Dbg(Debug::Pause));
        bindings.insert(Input::key(VirtualKeyCode::F9), KeyCat::Dbg(Debug::Run));
        bindings.insert(Input::key(VirtualKeyCode::F3), KeyCat::Dbg(Debug::Step));
        bindings.insert(Input::key(VirtualKeyCode::F4), KeyCat::Dbg(Debug::Reset));

        bindings.insert(Input::key(VirtualKeyCode::F5), KeyCat::Game(Shortcut::SaveState));
        bindings.insert(Input::key(VirtualKeyCode::F6), KeyCat::Game(Shortcut::LoadState));
        Self { bindings, inputs: Default::default(), modifiers: Default::default() }
    }
}

impl Keybindings {
    pub fn init(&mut self) {
        for (input, key) in &self.bindings {
            self.inputs.insert(*key, (*input, ElementState::Released));
        }
    }

    pub fn get(&self, key: KeyCat) -> Option<Input> {
        self.inputs.get(&key).map(|x| x.0)
    }

    fn set(&mut self, key: KeyCat, code: Input) {
        self.bindings.get(&code).map(|x| self.inputs.remove(x));
        self.inputs.get(&key).map(|(x, _)| self.bindings.remove(x));
        self.bindings.insert(code, key);
        self.inputs.insert(key, (code, ElementState::Released));
    }

    pub fn pressed(&self, key: KeyCat) -> bool {
        self.inputs.get(&key).map(|x| x.1 == ElementState::Pressed)
            .unwrap_or(false)
    }

    pub fn update_inputs(&mut self, event: &Event<Events>, proxy: &EventLoopProxy<Events>) {
        match event {
            Event::WindowEvent { event: WindowEvent::ModifiersChanged(state), .. } => {
                self.modifiers = *state;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(input), state, .. }, ..
                }, ..
            } => {
                let mut mods = self.modifiers;
                match input {
                    VirtualKeyCode::LShift | VirtualKeyCode::RShift => mods.remove(ModifiersState::SHIFT),
                    VirtualKeyCode::LControl | VirtualKeyCode::RControl => mods.remove(ModifiersState::CTRL),
                    VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => mods.remove(ModifiersState::ALT),
                    _ => {}
                }
                if let Some(&cat) = self.bindings.get(&Input::Keyboard(*input, mods))
                    .or_else(|| self.bindings.get(&Input::Keyboard(*input, ModifiersState::empty()))) {
                    proxy.send_event(if state == &ElementState::Pressed { Events::Press(cat) } else { Events::Release(cat) }).ok();
                    self.inputs.entry(cat).and_modify(|(_, x)| { *x = *state; });
                }
            }
            Event::DeviceEvent { event: DeviceEvent::Button { button, state }, .. } => {
                if let Some(&cat) = self.bindings.get(&Input::Device(*button)) {
                    proxy.send_event(if state == &ElementState::Pressed { Events::Press(cat) } else { Events::Release(cat) }).ok();
                    self.inputs.entry(cat).and_modify(|(_, x)| { *x = *state; });
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, joypad: &mut impl Joypad, event: &Event<Events>, io: &mut IORegs) {
        match event {
            Event::UserEvent(Events::Press(key @ KeyCat::Joy(..))) => joypad.update(*key, true, io),
            Event::UserEvent(Events::Release(key @ KeyCat::Joy(..))) => joypad.update(*key, false, io),
            _ => {}
        }
    }

    pub fn try_bind(&mut self, action: Option<KeyCat>, event: &Event<Events>) -> bool {
        match (action, event) {
            (Some(key), Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(input), state, .. }, ..
                }, ..
            }) if state == &ElementState::Released => {
                self.set(key, Input::Keyboard(*input, self.modifiers));
                true
            }
            (Some(key), Event::DeviceEvent {
                event: DeviceEvent::Button { button, state }, ..
            }) if state == &ElementState::Released => {
                self.set(key, Input::Device(*button));
                true
            }
            _ => { false }
        }
    }
}

pub trait Joypad {
    fn update(&mut self, key: KeyCat, pressed: bool, io: &mut IORegs);
}
