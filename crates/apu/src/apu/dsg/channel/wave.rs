use shared::io::{IO, IOReg};
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
    fn output(&self) -> f32 {
        todo!()
    }

    fn channel(&self) -> Channels { Channels::Wave }

    fn clock(&mut self) {
        todo!()
    }

    fn trigger(&mut self) {
        todo!()
    }

    fn length(&self) -> u8 {
        todo!()
    }
}
