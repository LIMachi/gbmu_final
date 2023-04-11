use shared::io::{IO, IOReg, IORegs};
use shared::mem::{Device, IOBus};
use crate::apu::dsg::channel::envelope::Envelope;
use super::{SoundChannel, Channels};

#[derive(Default)]
struct Registers {
    length: IOReg,
    volume: IOReg,
    freq: IOReg,
    ctrl: IOReg,
}

pub struct Channel {
    triggered: bool,
    buffer: u16,
    freq_timer: u16,
    envelope: Envelope,
    registers: Registers
}

impl Channel {
    pub fn new() -> Self {
        Self {
            triggered: false,
            buffer: 0xFFFF,
            freq_timer: 0,
            envelope: Envelope::default(),
            registers: Registers::default()
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
    fn update_envelope(&mut self) {
        self.envelope.update(self.registers.volume.value());
        self.registers.volume.reset_dirty();
    }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.registers.length = bus.io(IO::NR41);
        self.registers.volume = bus.io(IO::NR42);
        self.registers.freq = bus.io(IO::NR43);
        self.registers.ctrl = bus.io(IO::NR44);
    }
}

impl SoundChannel for Channel {
    fn output(&self) -> u8 {
        ((self.buffer & 1) as u8 ^ 1) * self.envelope.volume()
    }

    fn channel(&self) -> Channels { Channels::Noise }

    fn dac_enabled(&self) -> bool {
        self.registers.volume.value() & 0xF8 != 0
    }

    fn clock(&mut self) {
        if self.registers.volume.dirty() { self.update_envelope(); }
        if !self.triggered { return }
        if self.freq_timer == 0 {
            self.freq_timer = self.frequency();
            if self.freq_timer == 0 {
                self.triggered = false;
                return;
            }
            let mut r = self.buffer & 1;
            self.buffer >>= 1;
            r ^= self.buffer & 1;
            self.buffer |= r << 14;
            if self.registers.freq.bit(3) != 0 {
                self.buffer = (self.buffer & 0xFFBF) | (r << 6);
            }
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self) -> bool {
        self.triggered = true;
        self.buffer = 0xFFFF;
        self.envelope.trigger();
        self.freq_timer = self.frequency();
        false
    }

    fn envelope(&mut self) { self.envelope.clock(); }

    fn length(&self) -> u8 { 0x3F }
}
