use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use crate::apu::dsg::channel::envelope::Envelope;
use super::{SoundChannel, Channels};

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

#[derive(Default)]
struct Registers {
    sweep: IOReg,
    length: IOReg,
    volume: IOReg,
    wave1: IOReg,
    wave2: IOReg,
}

#[derive(Default)]
struct Sweep {
    shadow: u16,
    pace: u8,
    timer: u8,
    negate: bool,
    shift: u8,
    ran: bool,
    enabled: bool
}

impl Sweep {
    pub fn update(&mut self, data: u8) {
        self.pace = (data >> 4) & 0x7;
        self.timer = self.pace;
        self.negate = data & 0x4 != 0;
        self.shift = data & 0x7;
    }

    pub fn trigger(&mut self, freq: u16) -> u16 {
        self.shadow = freq;
        self.timer = self.pace;
        self.enabled = self.pace != 0 || self.shift != 0;
        self.ran = false;
        if self.shift != 0 { self.calc() } else { 0 }
    }

    pub fn calc(&mut self) -> u16 {
        self.ran = true;
        let f = self.shadow >> self.shift;
        if self.negate { self.shadow.wrapping_sub(f) } else { self.shadow + f }
    }

    pub fn tick(&mut self, wave1: &mut IOReg, wave2: &mut IOReg) -> bool {
        if !self.enabled { return false }
        self.timer -= 1;
        if self.timer == 0 {
            let f = self.calc();
            if f > 2047 { return true }
            if self.shift != 0 {
                wave1.direct_write(f as u8);
                wave2.direct_write((wave2.value() & 0x40) | (f >> 8) as u8);
            }
            self.timer = self.pace;
            if self.timer == 0 { self.timer = 8; }
            self.calc() > 2047
        } else {
            false
        }
    }
}

pub struct Channel {
    cycle: usize,
    freq_timer: u16,
    envelope: Envelope,
    sweep: Sweep,
    has_sweep: bool,
    registers: Registers,
}

impl Channel {
    pub fn new(sweep: bool) -> Self {
        Self {
            cycle: 0,
            freq_timer: 0,
            envelope: Envelope::default(),
            has_sweep: sweep,
            sweep: Sweep::default(),
            registers: Registers::default(),
        }
    }

    fn frequency(&self) -> u16 {
        self.registers.wave1.read() as u16 | ((self.registers.wave2.read() as u16 & 0x7) << 8)
    }

    fn update_sweep(&mut self) {
        if self.registers.sweep.dirty() {
            self.registers.sweep.reset_dirty();
            self.sweep.update(self.registers.sweep.value());
        }
    }

    fn update_envelope(&mut self) {
        if self.registers.volume.dirty() {
            self.envelope.update(self.registers.volume.value());
            self.registers.volume.reset_dirty();
        }
    }
}

impl SoundChannel for Channel {
    fn output(&self) -> u8 {
        let waveform = self.registers.length.value() >> 6;
        DUTY_CYCLES[waveform as usize][self.cycle] * self.envelope.volume()
    }

    fn channel(&self) -> Channels {
        if self.has_sweep { Channels::Sweep } else { Channels::Pulse }
    }

    fn dac_enabled(&self) -> bool {
        self.registers.volume.value() & 0xF8 != 0
    }

    fn clock(&mut self) {
        self.update_envelope();
        self.update_sweep();
        if self.freq_timer == 0 {
            self.cycle = (self.cycle + 1) & 0x7;
            self.freq_timer = 4 * (0x7FF - self.frequency());
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self) -> bool {
        self.envelope.trigger();
        self.freq_timer = 4 * (0x7FF - self.frequency()) | (self.freq_timer & 0x3);
        self.sweep.trigger(self.frequency()) > 2047
    }

    fn sweep(&mut self) -> bool {
        if self.has_sweep { self.sweep.tick(&mut self.registers.wave1, &mut self.registers.wave2) }
        else { false }
    }

    fn envelope(&mut self) {
        self.envelope.clock();
    }

    fn length(&self) -> u8 { 0x3F }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {
        if self.has_sweep {
            self.registers.sweep = bus.io(IO::NR10);
            self.registers.length = bus.io(IO::NR11);
            self.registers.volume = bus.io(IO::NR12);
            self.registers.wave1 = bus.io(IO::NR13);
            self.registers.wave2 = bus.io(IO::NR14);
        } else {
            self.registers.length = bus.io(IO::NR21);
            self.registers.volume = bus.io(IO::NR22);
            self.registers.wave1 = bus.io(IO::NR23);
            self.registers.wave2 = bus.io(IO::NR24);
        }
    }
}
