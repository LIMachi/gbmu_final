use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

mod wave;
mod pulse;
mod noise;
mod envelope;

pub use pulse::Channel as PulseChannel;
pub use wave::Channel as WaveChannel;
pub use noise::Channel as NoiseChannel;

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
    fn output(&self) -> u8;

    fn channel(&self) -> Channels;
    fn enable(&mut self) { }
    fn disable(&mut self) { }
    fn dac_enabled(&self) -> bool { false }

    fn clock(&mut self);
    fn trigger(&mut self) -> bool;
    fn sweep(&mut self) -> bool { false }
    fn envelope(&mut self) { }

    fn length(&self) -> u8;

    fn on_enable(&mut self) {  }
    fn on_disable(&mut self) {  }
}

impl SoundChannel for () {
    fn output(&self) -> u8 { 0 }
    fn channel(&self) -> Channels { Channels::Noise }
    fn clock(&mut self) { }
    fn trigger(&mut self) -> bool { false }
    fn length(&self) -> u8 { 0 }
}

// handles DAC output/switches/volume/length
pub(crate) struct Channel {
    pub enabled: bool,
    length_timer: u8,
    nr1: IOReg,
    nr4: IOReg,
    inner: Box<dyn SoundChannel + 'static>
}

impl Channel {
    pub fn new<C: SoundChannel + 'static>(channel: C) -> Self {
        Self {
            length_timer: 0,
            enabled: false,
            inner: Box::new(channel),
            nr1: IOReg::unset(),
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
    pub fn noise() -> Self { Self::new(NoiseChannel::new()) }

    pub fn enable(&mut self) { self.enabled = true; self.inner.on_enable(); }
    pub fn disable(&mut self) { self.enabled = false; self.inner.on_disable(); }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::Length if self.nr4.bit(6) != 0 && self.length_timer != 0 => {
                self.length_timer -= 1;
                if self.length_timer == 0 { self.enabled = false; }
            },
            Event::Sweep => if self.inner.sweep() { self.disable(); },
            Event::Envelope => self.inner.envelope(),
            _ => {}
        }
    }

    fn reload_length(&mut self) {
        let mask = self.inner.length();
        self.length_timer = (mask - (self.nr1.value() & mask)) + 1;
    }
}

impl SoundChannel for Channel {
    fn output(&self) -> u8 { self.inner.output() }

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
        if self.nr1.dirty() {
            self.nr1.reset_dirty();
            self.reload_length();
        }
        if !self.inner.dac_enabled() { self.disable(); }
        if self.enabled { self.inner.clock(); }
    }

    fn trigger(&mut self) -> bool {
        self.enable();
        if self.inner.trigger() { self.disable(); }
        self.reload_length();
        false
    }

    fn length(&self) -> u8 { 0 }
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
