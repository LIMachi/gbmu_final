use shared::io::{AccessMode, IO, IOReg};
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

    fn power_on(&mut self) { }
    fn power_off(&mut self) { }
}

impl SoundChannel for () {
    fn output(&self) -> u8 { 0 }
    fn channel(&self) -> Channels { Channels::Noise }
    fn clock(&mut self) { }
    fn trigger(&mut self) -> bool { false }
    fn length(&self) -> u8 { 0 }
}

pub struct Capacitor {
    factor: f32,
    value: f32
}

impl Capacitor {
    pub fn tick(&mut self, input: Option<f32>) -> f32 {
        if let Some(v) = input {
            self.value = v;
            v
        } else {
            self.value *= self.factor;
            // self.value = 0.0;
            self.value
        }
    }

    pub fn new(factor: f32) -> Self {
        Self { factor, value: 0. }
    }
}

// handles DAC output/switches/volume/length
pub(crate) struct Channel {
    pub enabled: bool,
    length_timer: u8,
    regs: [IO; 4],
    nr1: IOReg,
    nr2: IOReg,
    nr3: IOReg,
    nr4: IOReg,
    nr52: IOReg,
    pcm: IOReg,
    dac: Capacitor,
    inner: Box<dyn SoundChannel + 'static>
}

impl Channel {
    pub fn new<C: SoundChannel + 'static>(channel: C) -> Self {
        let s = channel.channel() as u16 * 5 + IO::NR11 as u16;
        let nr1 = IO::try_from(s).unwrap();
        let nr2 = IO::try_from(s + 1).unwrap();
        let nr3 = IO::try_from(s + 2).unwrap();
        let nr4 = IO::try_from(s + 3).unwrap();
        Self {
            length_timer: 0,
            enabled: false,
            inner: Box::new(channel),
            regs: [nr1, nr2, nr3, nr4],
            nr1: IOReg::unset(),
            nr2: IOReg::unset(),
            nr3: IOReg::unset(),
            dac: Capacitor::new(0.1),
            nr4: IOReg::unset(),
            nr52: IOReg::unset(),
            pcm: IOReg::unset(),
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

    pub fn power_on(&mut self) {
        self.nr1.set_access(self.regs[0].access());
        self.nr2.set_access(self.regs[1].access());
        self.nr3.set_access(self.regs[2].access());
        self.nr4.set_access(self.regs[3].access());
        self.inner.power_on();
    }

    pub fn power_off(&mut self) {
        self.nr1.set_access(AccessMode::rdonly()).direct_write(0); //FIXME: DMG allow length modification!
        self.nr2.set_access(AccessMode::rdonly()).direct_write(0);
        self.nr3.set_access(AccessMode::rdonly()).direct_write(0);
        self.nr4.set_access(AccessMode::rdonly()).direct_write(0);
        self.inner.power_off();
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.nr52.set(self.inner.channel() as u8);
        self.inner.on_enable();
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.nr52.reset(self.inner.channel() as u8);
        self.inner.on_disable();
    }

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

    pub fn output(&mut self, cgb: bool) -> f32 {
        let out = if self.enabled { self.inner.output() } else { 0 };
        if cgb {
            let n = (self.inner.channel() as u8) & 1;
            let t = self.pcm.value() & (0xF0 >> (4 * n));
            let p = t | ((out & 0xF) << (4 * n));
            self.pcm.direct_write(p);
        }
        self.dac.tick(if self.dac_enabled() { Some(1. - out as f32 / 7.5) } else { None })
    }

    pub fn channel(&self) -> Channels { self.inner.channel() }
    pub fn dac_enabled(&self) -> bool {
        self.inner.dac_enabled()
    }

    pub fn clock(&mut self) {
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

    pub fn trigger(&mut self) -> bool {
        self.enable();
        if self.inner.trigger() { self.disable(); }
        self.reload_length();
        false
    }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.nr1 = bus.io(match self.inner.channel() {
            Channels::Sweep => IO::NR11,
            Channels::Pulse => IO::NR21,
            Channels::Wave => IO::NR31,
            Channels::Noise => IO::NR41
        });
        self.nr2 = bus.io(match self.inner.channel() {
            Channels::Sweep => IO::NR12,
            Channels::Pulse => IO::NR22,
            Channels::Wave => IO::NR32,
            Channels::Noise => IO::NR42
        });
        self.nr3 = bus.io(match self.inner.channel() {
            Channels::Sweep => IO::NR13,
            Channels::Pulse => IO::NR23,
            Channels::Wave => IO::NR33,
            Channels::Noise => IO::NR43
        });
        self.nr4 = bus.io(match self.inner.channel() {
            Channels::Sweep => IO::NR14,
            Channels::Pulse => IO::NR24,
            Channels::Wave => IO::NR34,
            Channels::Noise => IO::NR44
        });
        self.inner.configure(bus);
        self.pcm = bus.io(match self.inner.channel() {
            Channels::Sweep | Channels::Pulse => IO::PCM12,
            Channels::Wave | Channels::Noise => IO::PCM34,
        });
        self.nr52 = bus.io(IO::NR52);
    }
}
