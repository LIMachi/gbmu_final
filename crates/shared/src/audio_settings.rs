use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioSettings {
    pub volume: f32,
    pub channels: [bool; 4]
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 1f32,
            channels: [true, true, true, true]
        }
    }
}
