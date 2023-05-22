extern crate core;

use cpal::traits::DeviceTrait;
use rodio::cpal;
use serde::{Deserialize, Serialize};

pub use apu::Apu;
use driver::{Audio, Input};

mod apu;
mod driver;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    dev_name: String,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            dev_name: driver::default_device()
        }
    }
}

pub struct Controller {
    driver: Audio,
}

impl Controller {
    pub fn devices() -> impl Iterator<Item=String> {
        Audio::devices().filter_map(|x| x.name().ok())
    }

    pub fn device(&self) -> &String {
        self.driver.device()
    }

    pub fn config(&self) -> SoundConfig {
        SoundConfig {
            dev_name: self.driver.device().clone()
        }
    }

    pub fn switch<S: Into<String>>(&mut self, name: S, apu: &mut Apu) {
        match self.driver.switch(name) {
            Ok(x) => apu.switch(x.sample_rate(), x.bind()),
            Err(e) => log::warn!("failed to switch device: {e:?}")
        }
    }

    pub fn new(config: &SoundConfig) -> Self {
        let audio = Audio::new(config);
        Self { driver: audio }
    }

    pub fn apu(&self, cgb: bool) -> Apu {
        Apu::new(self.driver.sample_rate(), self.driver.bind(), cgb)
    }

    pub fn reload(&mut self, apu: &mut Apu) {
        apu.input = self.driver.bind();
    }
}
