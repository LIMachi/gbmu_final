use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use winit::event::VirtualKeyCode;

use crate::utils::Cell;

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
    Reset,
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyCat {
    Joy(Keys),
    Dbg(Shortcut),
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

    pub fn shortcuts() -> impl IntoIterator<Item=KeyCat> {
        [KeyCat::Dbg(Shortcut::Pause),
            KeyCat::Dbg(Shortcut::Run),
            KeyCat::Dbg(Shortcut::Step),
            KeyCat::Dbg(Shortcut::Reset),
        ]
    }
}

#[derive(Clone)]
pub struct Keybindings {
    bindings: Rc<RefCell<HashMap<VirtualKeyCode, KeyCat>>>,
}

impl Serialize for Keybindings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("Keybindings", 1)?;
        state.serialize_field("bindings", &*self.bindings.as_ref().borrow())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Keybindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Bindings }

        struct KeybindingsVisitor;
        impl<'de> Visitor<'de> for KeybindingsVisitor {
            type Value = Keybindings;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result { formatter.write_str("struct Keybindings") }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let bindings: HashMap<VirtualKeyCode, KeyCat> =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(Keybindings { bindings: bindings.cell() })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut bindings: Option<HashMap<VirtualKeyCode, KeyCat>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Bindings => {
                            if bindings.is_some() {
                                return Err(de::Error::duplicate_field("bindings"));
                            }
                            bindings = Some(map.next_value()?);
                        }
                    }
                }
                let bindings = bindings.ok_or_else(|| de::Error::missing_field("bindings"))?;
                Ok(Keybindings { bindings: bindings.cell() })
            }
        }
        const FIELDS: &'static [&'static str] = &["bindings"];
        deserializer.deserialize_struct("Keybindings", FIELDS, KeybindingsVisitor)
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(VirtualKeyCode::A, KeyCat::Joy(Keys::Left));
        bindings.insert(VirtualKeyCode::D, KeyCat::Joy(Keys::Right));
        bindings.insert(VirtualKeyCode::S, KeyCat::Joy(Keys::Down));
        bindings.insert(VirtualKeyCode::W, KeyCat::Joy(Keys::Up));

        bindings.insert(VirtualKeyCode::Space, KeyCat::Joy(Keys::A));
        bindings.insert(VirtualKeyCode::LShift, KeyCat::Joy(Keys::B));
        bindings.insert(VirtualKeyCode::Z, KeyCat::Joy(Keys::Start));
        bindings.insert(VirtualKeyCode::X, KeyCat::Joy(Keys::Select));

        bindings.insert(VirtualKeyCode::F2, KeyCat::Dbg(Shortcut::Pause));
        bindings.insert(VirtualKeyCode::F9, KeyCat::Dbg(Shortcut::Run));
        bindings.insert(VirtualKeyCode::F3, KeyCat::Dbg(Shortcut::Step));
        bindings.insert(VirtualKeyCode::F4, KeyCat::Dbg(Shortcut::Reset));
        Self { bindings: bindings.cell() }
    }
}

impl Keybindings {
    pub fn set(&mut self, key: KeyCat, code: VirtualKeyCode) {
        self.bindings.as_ref().borrow_mut().drain_filter(|_v, x| &key == x);
        self.bindings.as_ref().borrow_mut().insert(code, key);
    }

    pub fn get(&self, key: VirtualKeyCode) -> Option<KeyCat> {
        self.bindings.as_ref().borrow().get(&key).map(|x| *x)
    }

    pub fn has(&self, key: KeyCat) -> Option<VirtualKeyCode> {
        self.bindings.as_ref().borrow().iter().find(|(_v, s)| s == &&key).map(|(v, _)| *v)
    }
}
