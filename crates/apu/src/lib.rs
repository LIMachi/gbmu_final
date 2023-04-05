extern crate core;

use std::cell::RefCell;
use std::fmt::Formatter;
use std::rc::Rc;

use rodio::cpal;
use cpal::traits::DeviceTrait;
use shared::utils::Cell;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;

mod apu;
mod driver;

pub use apu::Apu;
use driver::{Audio, Input};
use shared::audio_settings::AudioSettings;

#[derive(Clone)]
pub struct SoundConfig {
    dev_name: Rc<RefCell<String>>
}

impl Serialize for SoundConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("SoundConfig", 1)?;
        state.serialize_field("dev_name", &*self.dev_name.borrow())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SoundConfig {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { DevName }

        struct SoundConfigVisitor;
        impl<'de> Visitor<'de> for SoundConfigVisitor {
            type Value = SoundConfig;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct SoundConfig")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                Ok(SoundConfig{dev_name:seq.next_element::<String>()?.ok_or_else(|| de::Error::invalid_length(0, &self))?.cell()})
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut dev_name = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::DevName => {
                            if dev_name.is_some() {
                                return Err(de::Error::duplicate_field("dev_name"));
                            }
                            dev_name = Some(map.next_value::<String>()?.cell());
                        }
                    }
                }
                let dev_name = dev_name.ok_or_else(|| de::Error::missing_field("dev_name"))?;
                Ok(SoundConfig{dev_name})
            }
        }
        const FIELDS: &'static [&'static str] = &["dev_name"];
        deserializer.deserialize_struct("SoundConfig", FIELDS, SoundConfigVisitor)
    }
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            dev_name: driver::default_device().cell()
        }
    }
}

#[derive(Clone)]
pub struct Controller {
    input: Input,
    driver: Rc<RefCell<Audio>>
}

impl Controller {
    pub fn devices() -> impl Iterator<Item = String> {
        Audio::devices().filter_map(|x| x.name().ok())
    }

    pub fn device(&self) -> String {
        self.driver.as_ref().borrow().device()
    }

    pub fn switch<S: Into<String>>(&mut self, name: S) {
        self.driver.as_ref().borrow_mut()
            .switch(name)
            .map(|x| x.bind(&mut self.input)).ok();
    }

    pub fn new(config: &SoundConfig) -> Self {
        let audio = Audio::new(config);
        let mut input = Input::default();
        audio.bind(&mut input);
        Self { input, driver: audio.cell() }
    }

    pub fn apu(&self, settings: AudioSettings) -> Apu {
        Apu::new(self.driver.as_ref().borrow().sample_rate(), self.input.clone(), settings)
    }
}
