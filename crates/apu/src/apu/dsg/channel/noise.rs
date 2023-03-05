use shared::mem::{Device, IOBus};
use super::{SoundChannel, Channels};

pub struct Channel {

}

impl Channel {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {

    }
}

impl SoundChannel for Channel {
    fn output(&self) -> u8 {
        todo!()
    }

    fn channel(&self) -> Channels { Channels::Noise }

    fn clock(&mut self) {
        todo!()
    }

    fn trigger(&mut self) -> bool { false }

    fn length(&self) -> u8 {
        0x3F
    }
}
