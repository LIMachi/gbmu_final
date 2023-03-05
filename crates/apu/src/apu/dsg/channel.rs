use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

mod wave;
mod pulse;
mod noise;
mod envelope;
use envelope::Envelope;

use pulse::Channel as PulseChannel;
use wave::Channel as WaveChannel;
use noise::Channel as NoiseChannel;

pub enum Event {
    Envelope,
    Sweep,
    Length
}

#[repr(u8)]
pub(crate) enum Channels {
    Sweep = 0,
    Pulse = 1,
    Wave = 2,
    Noise = 3
}

// inner waveform generator
pub(crate) trait SoundChannel: Device {
    fn output(&self) -> f32;

    fn channel(&self) -> Channels;
    fn enable(&mut self) { }
    fn disable(&mut self) { }
    fn dac_enabled(&self) -> bool { false }

    fn clock(&mut self);
    fn trigger(&mut self);
    fn sweep(&mut self) -> bool { false }
    fn envelope(&mut self) { }

    fn length(&self) -> u8;
}

impl SoundChannel for () {
    fn output(&self) -> f32 { 0. }
    fn channel(&self) -> Channels { Channels::Noise }
    fn clock(&mut self) { }
    fn trigger(&mut self) {}
    fn length(&self) -> u8 { 0 }
}

// handles DAC output/switches/volume/length
pub(crate) struct Channel {
    enabled: bool,
    length_timer: u8,
    nr4: IOReg,
    inner: Box<dyn SoundChannel + 'static>
}

impl Channel {
    pub fn new<C: SoundChannel + 'static>(channel: C) -> Self {
        Self {
            length_timer: 0,
            enabled: false,
            inner: Box::new(channel),
            nr4: IOReg::unset(),
        }
    }

    pub fn sweep() -> Self {
        Self::new(PulseChannel::new(true))
    }
    pub fn pulse() -> Self {
        Self::new(PulseChannel::new(false))
    }
    pub fn wave() -> Self {
        Self::new(WaveChannel::new())
    }
    pub fn noise() -> Self {
        Self::new(NoiseChannel::new())
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Length if self.nr4.bit(6) != 0 => {
                if self.length_timer != 0 {
                    self.length_timer -= 1;
                    self.enabled = false;
                }
            },
            Event::Sweep => if self.inner.sweep() { self.disable(); },
            Event::Envelope => self.inner.envelope(),
            _ => {}
        }
    }
}

impl SoundChannel for Channel {
    fn output(&self) -> f32 {
        0.
    }

    fn channel(&self) -> Channels {
        self.inner.channel()
    }

    fn dac_enabled(&self) -> bool {
        self.inner.dac_enabled()
    }

    fn clock(&mut self) {
        if self.nr4.dirty() {
            self.nr4.reset_dirty();
            if self.nr4.bit(7) != 0 { self.trigger(); }
        }
        self.inner.clock();
        if !self.inner.dac_enabled() { self.disable(); }
    }

    fn trigger(&mut self) {
        self.inner.trigger();
        self.enable();
        self.length_timer = self.inner.length();
    }

    fn length(&self) -> u8 {
        self.inner.length()
    }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.nr4 = bus.io(match self.inner.channel() {
            Channels::Sweep => IO::NR14,
            Channels::Pulse => IO::NR24,
            Channels::Wave => IO::NR34,
            Channels::Noise => IO::NR44
        });
        self.inner.configure(bus);
    }
}
