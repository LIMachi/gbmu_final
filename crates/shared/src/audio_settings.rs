use std::cell::RefCell;
use std::fmt::Formatter;
use std::rc::Rc;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use crate::utils::Cell;

#[derive(Clone)]
pub struct AudioSettings {
    pub volume: Rc<RefCell<f32>>,
    pub channels: [Rc<RefCell<bool>>; 4]
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 1f32.cell(),
            channels: [true.cell(), true.cell(), true.cell(), true.cell()]
        }
    }
}

impl Serialize for AudioSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("AudioSettings", 1)?;
        state.serialize_field("volume", &*self.volume.as_ref().borrow())?;
        println!("channels: {:?}", self.channels);
        let mut c = [true, true, true, true];
        for i in 0..4 {
            c[i] = *self.channels[i].as_ref().borrow();
        }
        println!("c: {:?}", c);
        state.serialize_field("channels", &c)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AudioSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Volume, Channels }

        struct AudioSettingsVisitor;
        impl<'de> Visitor<'de> for AudioSettingsVisitor {
            type Value = AudioSettings;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct AudioSettings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let volume = seq.next_element::<f32>()?.ok_or_else(|| de::Error::invalid_length(0, &self))?.cell();
                let c = seq.next_element::<[bool; 4]>()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let channels = [true.cell(), true.cell(), true.cell(), true.cell()];
                for i in 0..4 {
                    channels[i].replace(c[i]);
                }
                Ok(AudioSettings{volume, channels})
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut volume = None;
                let mut channels = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Volume => {
                            if volume.is_some() {
                                return Err(de::Error::duplicate_field("volume"));
                            }
                            volume = Some(map.next_value::<f32>()?.cell());
                        },
                        Field::Channels => {
                            if channels.is_some() {
                                return Err(de::Error::duplicate_field("channels"));
                            }
                            let c = map.next_value::<[bool; 4]>()?;
                            let t = [true.cell(), true.cell(), true.cell(), true.cell()];
                            for i in 0..4 {
                                t[i].replace(c[i]);
                            }
                            channels = Some(t);
                        }
                    }
                }
                let volume = volume.ok_or_else(|| de::Error::missing_field("volume"))?;
                let channels = channels.ok_or_else(|| de::Error::missing_field("channels"))?;
                Ok(AudioSettings{volume, channels})
            }
        }

        const FIELDS: &'static [&'static str] = &["volume", "channels"];
        deserializer.deserialize_struct("AudioSettings", FIELDS, AudioSettingsVisitor)
    }
}