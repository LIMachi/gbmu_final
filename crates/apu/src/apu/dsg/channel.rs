use std::fmt::Formatter;
use dyn_clone::DynClone;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use shared::io::{AccessMode, IO, IODevice, IORegs};

mod wave;
mod pulse;
mod noise;
mod envelope;

pub use pulse::Channel as PulseChannel;
pub use wave::Channel as WaveChannel;
pub use noise::Channel as NoiseChannel;
use shared::mem::IOBus;

pub enum Event {
    Envelope,
    Sweep,
    Length
}

#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum Channels {
    Sweep = 0,
    Pulse = 1,
    Wave = 2,
    Noise = 3
}

// inner waveform generator
pub(crate) trait SoundChannel: IODevice + DynClone {
    fn output(&self, io: &mut IORegs) -> u8;

    fn channel(&self) -> Channels;
    fn enable(&mut self) { }
    fn disable(&mut self) { }
    fn dac_enabled(&self) -> bool { false }

    fn clock(&mut self, io: &mut IORegs);
    fn trigger(&mut self, io: &mut IORegs) -> bool;
    fn sweep(&mut self, _io: &mut IORegs) -> bool { false }
    fn envelope(&mut self) { }

    fn length(&self) -> u8;

    fn on_enable(&mut self, _io: &mut IORegs) {  }
    fn on_disable(&mut self, _io: &mut IORegs) {  }

    fn power_on(&mut self, _io: &mut IORegs) { }
    fn power_off(&mut self, _io: &mut IORegs) { }

    fn raw(&self) -> Vec<u8> { vec![] }
}

impl SoundChannel for () {
    fn output(&self, _io: &mut IORegs) -> u8 { 0 }
    fn channel(&self) -> Channels { Channels::Noise }
    fn clock(&mut self, _io: &mut IORegs) { }
    fn trigger(&mut self, _io: &mut IORegs) -> bool { false }
    fn length(&self) -> u8 { 0 }
}

#[derive(Serialize, Deserialize, Clone)]
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
#[derive(Serialize, Deserialize)]
pub(crate) struct Channel {
    pub enabled: bool,
    length_timer: u8,
    nr1: IO,
    nr2: IO,
    nr3: IO,
    nr4: IO,
    dac: Capacitor,
    inner: Box<dyn SoundChannel + 'static>,
    pcm: IO
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            length_timer: self.length_timer,
            nr1: self.nr1,
            nr2: self.nr2,
            nr3: self.nr3,
            nr4: self.nr4,
            dac: self.dac.clone(),
            inner: dyn_clone::clone_box(&*self.inner),
            pcm: self.pcm
        }
    }
}

#[derive(Serialize, Deserialize)]
struct InnerSoundChannel {
    kind: Channels,
    raw: Vec<u8>
}

impl Serialize for Box<dyn SoundChannel + 'static> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let kind = self.channel();
        let raw = self.raw();
        InnerSoundChannel{kind, raw}.serialize(serializer)
    }
}

impl <'de> Deserialize<'de> for Box<dyn SoundChannel + 'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Deserialize::deserialize(deserializer).map(|InnerSoundChannel{kind, raw}| {
            match kind {
                Channels::Sweep => PulseChannel::from_raw(true, raw),
                Channels::Pulse => PulseChannel::from_raw(false, raw),
                Channels::Wave => WaveChannel::from_raw(raw),
                Channels::Noise => NoiseChannel::from_raw(raw)
            }
        })
    }
}

impl Channel {
    pub fn new<C: SoundChannel + 'static>(channel: C) -> Self {
        let c = channel.channel() as u16;
        let s = c * 5 + IO::NR11 as u16;
        let nr1 = IO::try_from(s).unwrap();
        let nr2 = IO::try_from(s + 1).unwrap();
        let nr3 = IO::try_from(s + 2).unwrap();
        let nr4 = IO::try_from(s + 3).unwrap();
        Self {
            length_timer: 0,
            enabled: false,
            inner: Box::new(channel),
            nr1,
            nr2,
            nr3,
            nr4,
            dac: Capacitor::new(0.1),
            pcm: match c {
                0 | 1 => IO::PCM12,
                _ => IO::PCM34,
            }
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

    pub fn power_on(&mut self, io: &mut IORegs) {
        io.io_mut(self.nr1).set_access(self.nr1.access());
        io.io_mut(self.nr2).set_access(self.nr2.access());
        io.io_mut(self.nr3).set_access(self.nr3.access());
        io.io_mut(self.nr4).set_access(self.nr4.access());
        self.inner.power_on(io);
    }

    pub fn power_off(&mut self, io: &mut IORegs) {
       io.io_mut(self.nr1).direct_write(0).set_access(AccessMode::rdonly()); //FIXME: DMG allow length modification!
       io.io_mut(self.nr2).direct_write(0).set_access(AccessMode::rdonly());
       io.io_mut(self.nr3).direct_write(0).set_access(AccessMode::rdonly());
       io.io_mut(self.nr4).direct_write(0).set_access(AccessMode::rdonly());
        self.inner.power_off(io);
    }

    pub fn enable(&mut self, io: &mut IORegs) {
        self.enabled = true;
        io.io_mut(IO::NR52).set(self.inner.channel() as u8);
        self.inner.on_enable(io);
    }

    pub fn disable(&mut self, io: &mut IORegs) {
        if self.enabled {
            self.enabled = false;
            io.io_mut(IO::NR52).reset(self.inner.channel() as u8);
            self.inner.on_disable(io);
        }
    }

    pub fn event(&mut self, event: Event, io: &mut IORegs) {
        match event {
            Event::Length if io.io(self.nr4).bit(6) != 0 && self.length_timer != 0 => {
                self.length_timer -= 1;
                if self.length_timer == 0 { self.enabled = false; }
            },
            Event::Sweep => if self.inner.sweep(io) { self.disable(io); },
            Event::Envelope => self.inner.envelope(),
            _ => {}
        }
    }

    fn reload_length(&mut self, v: u8) {
        let mask = self.inner.length();
        self.length_timer = (mask - (v & mask)) + 1;
    }

    pub fn output(&mut self, cgb: bool, io: &mut IORegs) -> f32 {
        let out = if self.enabled { self.inner.output(io) } else { 0 };
        if cgb {
            let n = (self.inner.channel() as u8) & 1;
            let t = io.io(self.pcm).value() & (0xF0 >> (4 * n));
            let p = t | ((out & 0xF) << (4 * n));
            io.io_mut(self.pcm).direct_write(p);
        }
        self.dac.tick(if self.dac_enabled() { Some(1. - out as f32 / 7.5) } else { None })
    }

    pub fn channel(&self) -> Channels { self.inner.channel() }
    pub fn dac_enabled(&self) -> bool {
        self.inner.dac_enabled()
    }

    pub fn clock(&mut self, io: &mut IORegs) {
        if !self.inner.dac_enabled() {
            self.disable(io);
        } else if self.enabled {
            self.inner.clock(io);
        }
    }

    pub fn trigger(&mut self, io: &mut IORegs) {
        self.enable(io);
        if self.inner.trigger(io) { self.disable(io); }
        self.reload_length(io.io(self.nr1).value());
    }
}

impl IODevice for Channel {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        self.inner.write(io, v, bus);
        if io == self.nr4 && v & 0x80 != 0 { self.trigger(bus.io_regs()); }
        if io == self.nr1 { self.reload_length(v); }
    }
}
