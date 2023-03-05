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
    fn output(&self) -> u8 {
        todo!()
    }

    fn on_disable(&mut self) {
        // TODO enable wave ram access
    }

    fn on_enable(&mut self) {
        // TODO disable wave ram access
    }

    fn channel(&self) -> Channels { Channels::Wave }

    fn clock(&mut self) {
        todo!()
    }

    fn trigger(&mut self) -> bool { false }

    fn length(&self) -> u8 { 0xFF }
}
