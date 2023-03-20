use std::cell::RefCell;
use serde::{Serialize, Deserialize, Serializer, Deserializer, de};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;
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
            Section::Joy(Keys::Select)
        ]
    }

    pub fn shortcuts() -> impl IntoIterator<Item = Section> {
        [Section::Dbg(Shortcut::Pause),
            Section::Dbg(Shortcut::Run),
            Section::Dbg(Shortcut::Step),
            Section::Dbg(Shortcut::Reset),
        ]
    }
}

#[derive(Clone)]
pub struct Keybindings {
    bindings: Rc<RefCell<HashMap<VirtualKeyCode, Section>>>,
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
                let bindings: HashMap<VirtualKeyCode, Section> =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(Keybindings { bindings: bindings.cell() })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut bindings: Option<HashMap<VirtualKeyCode, Section>> = None;
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
        bindings.insert(VirtualKeyCode::F4, Section::Dbg(Shortcut::Reset));
        Self { bindings: bindings.cell() }
    }
}

impl Keybindings {
    pub fn set(&mut self, key: Section, code: VirtualKeyCode) {
        self.bindings.as_ref().borrow_mut().drain_filter(|_v , x| &key == x);
        self.bindings.as_ref().borrow_mut().insert(code, key);
    }

    pub fn get(&self, key: VirtualKeyCode) -> Option<Section> {
        self.bindings.as_ref().borrow().get(&key).map(|x| *x)
    }

    pub fn has(&self, key: Section) -> Option<VirtualKeyCode> {
        self.bindings.as_ref().borrow().iter().find(|(_v, s)| s == &&key ).map(|(v, _)| *v )
    }
}
