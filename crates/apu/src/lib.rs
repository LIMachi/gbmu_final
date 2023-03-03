use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;

use rodio::{cpal, Source};
use cpal::{traits::{HostTrait, DeviceTrait}};
use shared::mem::IOBus;
use shared::utils::Cell;
use serde::{Deserialize, Serialize};

mod apu;
mod driver;

pub use apu::Apu;
use driver::{Audio, Input};

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundConfig {
    #[serde(default = "default_device")]
    dev_name: String
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            dev_name: default_device()
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

    pub fn switch<S: Into<String>>(&mut self, name: S) {
        self.driver.as_ref().borrow_mut().switch(name).map(|x| {
            self.input.replace(x.bind());
        }).ok();
    }

    pub fn new(config: &SoundConfig) -> Self {
        let audio = Audio::new(config);
        let input = audio.bind();
        Self { input, driver: audio.cell() }
    }

    pub fn apu(&self) -> Apu {
        Apu::new(self.driver.as_ref().borrow().sample_rate(), self.input.clone())
    }
}
