use shared::io::{IO, IORegs};
use shared::mem::Device;
use crate::apu::dsg::channel::envelope::Envelope;
use super::{SoundChannel, Channels};

pub struct Channel {
    triggered: bool,
    buffer: u16,
    freq_timer: u16,
    envelope: Envelope,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            triggered: false,
            buffer: 0xFFFF,
            freq_timer: 0,
            envelope: Envelope::default(),
        }
    }

    fn frequency(&self, io: &mut IORegs) -> u16 {
        let f = io.io(IO::NR43).value() as u16;
        let r = f & 0x7;
        let s = f >> 4;
        match (r, s) {
            (0, 0) => 8,
            (0, s) => 16 << (s - 1),
            (r, s) => 16 * (r << s)
        }
    }
}

impl Device for Channel {}

impl SoundChannel for Channel {
    fn output(&self, _io: &mut IORegs) -> u8 {
        ((self.buffer & 1) as u8 ^ 1) * self.envelope.volume()
    }

    fn channel(&self) -> Channels { Channels::Noise }

    fn dac_enabled(&self, io: &mut IORegs) -> bool {
        io.io(IO::NR42).value() & 0xF8 != 0
    }

    fn clock(&mut self, io: &mut IORegs) {
        let volume = io.io(IO::NR42);
        if volume.dirty() {
            self.envelope.update(volume.value());
            volume.reset_dirty();
        }
        if !self.triggered { return }
        if self.freq_timer == 0 {
            self.freq_timer = self.frequency(io);
            if self.freq_timer == 0 {
                self.triggered = false;
                return;
            }
            let mut r = self.buffer & 1;
            self.buffer >>= 1;
            r ^= self.buffer & 1;
            self.buffer |= r << 14;
            if io.io(IO::NR43).bit(3) != 0 {
                self.buffer = (self.buffer & 0xFFBF) | (r << 6);
            }
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self, io: &mut IORegs) -> bool {
        self.triggered = true;
        self.buffer = 0xFFFF;
        self.envelope.trigger();
        self.freq_timer = self.frequency(io);
        false
    }

    fn envelope(&mut self) { self.envelope.clock(); }

    fn length(&self) -> u8 { 0x3F }
}
