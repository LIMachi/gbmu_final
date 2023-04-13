extern crate core;
use rodio::cpal;
use cpal::traits::DeviceTrait;
use serde::{Deserialize, Serialize};

mod apu;
mod driver;

pub use apu::Apu;
use driver::{Audio, Input};
use shared::audio_settings::AudioSettings;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    dev_name: String
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            dev_name: driver::default_device()
        }
    }
}

pub struct Controller {
    input: Input,
    driver: Audio
}

impl Controller {
    pub fn devices() -> impl Iterator<Item = String> {
        Audio::devices().filter_map(|x| x.name().ok())
    }

    pub fn device(&self) -> &String {
        self.driver.device()
    }

    pub fn switch<S: Into<String>>(&mut self, name: S) {
        self.driver
            .switch(name)
            .map(|x| x.bind(&mut self.input)).ok();
    }

    pub fn new(config: &SoundConfig) -> Self {
        let audio = Audio::new(config);
        let mut input = Input::default();
        audio.bind(&mut input);
        Self { input, driver: audio }
    }

    pub fn apu(&self, settings: AudioSettings) -> Apu {
        Apu::new(self.driver.sample_rate(), self.input.clone(), settings)
    }
}
