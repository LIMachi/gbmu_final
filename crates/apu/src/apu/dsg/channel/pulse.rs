use shared::io::{AccessMode, IO, IORegs};
use shared::mem::{Device, IOBus};
use crate::apu::dsg::channel::envelope::Envelope;
use super::{SoundChannel, Channels};

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

struct Registers {
    length: IO,
    volume: IO,
    wave1: IO,
    wave2: IO,
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
        let neg = self.negate;

        self.pace = (data >> 4) & 0x7;
        self.timer = if self.pace != 0 { self.pace } else { 8 };
        self.negate = data & 0x8 != 0;
        self.shift = data & 0x7;
        if neg && !self.negate && self.ran { self.enabled = false; }
        self.ran = false;
    }

    pub fn trigger(&mut self, freq: u16) -> u16 {
        self.shadow = freq;
        self.timer = if self.pace != 0 { self.pace } else { 8 };
        self.enabled = self.pace != 0 || self.shift != 0;
        self.ran = false;
        if self.shift != 0 { self.calc() } else { 0 }
    }

    pub fn calc(&mut self) -> u16 {
        self.ran = true;
        let f = self.shadow >> self.shift;
        if self.negate { self.shadow.wrapping_sub(f) } else { self.shadow + f }
    }

    pub fn tick(&mut self, io: &mut IORegs, wave1: IO, wave2: IO) -> bool {
        if !self.enabled { return false }
        self.timer -= 1;
        if self.timer == 0 {
            self.timer = self.pace;
            if self.timer == 0 { self.timer = 8 }
            if self.enabled && self.pace != 0 {
                let f = self.calc();
                if f <= 2047 && self.shift != 0 {
                    io.io(wave1).direct_write(f as u8);
                    let wave2 = io.io(wave2);
                    wave2.direct_write((wave2.value() & 0x40) | ((f & 0x7FF) >> 8) as u8);
                    self.shadow = f;
                }
                f > 2047 || self.calc() > 2047
            } else { false }
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
    triggered: bool
}

impl Channel {
    pub fn new(sweep: bool) -> Self {
        Self {
            triggered: false,
            cycle: 0,
            freq_timer: 0,
            envelope: Envelope::default(),
            has_sweep: sweep,
            sweep: Sweep::default(),
            registers: if sweep {
                Registers {
                    length: IO::NR11,
                    volume: IO::NR12,
                    wave1: IO::NR13,
                    wave2: IO::NR14
                }
            } else {
                Registers {
                    length: IO::NR21,
                    volume: IO::NR22,
                    wave1: IO::NR23,
                    wave2: IO::NR24
                }
            },
        }
    }

    fn frequency(&self, io: &mut IORegs) -> u16 {
        io.io(self.registers.wave1).value() as u16 | ((io.io(self.registers.wave2).value() as u16 & 0x7) << 8)
    }

    fn update_sweep(&mut self, io: &mut IORegs) {
        io.io(IO::NR10).reset_dirty();
        self.sweep.update(io.io(IO::NR10).value());
    }

    fn update_envelope(&mut self, io: &mut IORegs) {
        self.envelope.update(io.io(self.registers.volume).value());
        io.io(self.registers.volume).reset_dirty();
    }
}

impl SoundChannel for Channel {
    fn output(&self, io: &mut IORegs) -> u8 {
        let waveform = io.io(self.registers.length).value() >> 6;
        DUTY_CYCLES[waveform as usize][self.cycle] * self.envelope.volume()
    }

    fn channel(&self) -> Channels {
        if self.has_sweep { Channels::Sweep } else { Channels::Pulse }
    }

    fn dac_enabled(&self, io: &mut IORegs) -> bool {
        io.io(self.registers.volume).value() & 0xF8 != 0
    }

    fn clock(&mut self, io: &mut IORegs) {
        if io.io(self.registers.volume).dirty() { self.update_envelope(io); }
        if self.has_sweep && io.io(IO::NR10).dirty() { self.update_sweep(io); }
        if !self.triggered { return }
        if self.freq_timer == 0 {
            self.cycle = (self.cycle + 1) & 0x7;
            self.freq_timer = 4 * (0x7FF - self.frequency(io));
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self, io: &mut IORegs) -> bool {
        self.triggered = true;
        self.envelope.trigger();
        self.freq_timer = 4 * (0x7FF - self.frequency(io)) | (self.freq_timer & 0x3);
        self.sweep.trigger(self.frequency(io)) > 2047
    }

    fn sweep(&mut self, io: &mut IORegs) -> bool {
        if self.has_sweep { self.sweep.tick(io, self.registers.wave1, self.registers.wave2) }
        else { false }
    }

    fn envelope(&mut self) {
        self.envelope.clock();
    }

    fn length(&self) -> u8 { 0x3F }

    fn power_on(&mut self, io: &mut IORegs) {
        if self.has_sweep {
            io.io(IO::NR10).set_access(IO::NR10.access());
        }
    }

    fn power_off(&mut self, io: &mut IORegs) {
        if self.has_sweep {
            io.io(IO::NR10).direct_write(0).set_access(AccessMode::rdonly());
        }
    }
}

impl Device for Channel {
    fn configure(&mut self, _bus: &dyn IOBus) {
        // if self.has_sweep {
        //     self.registers.sweep = bus.io(IO::NR10);
        //     self.registers.length = bus.io(IO::NR11);
        //     self.registers.volume = bus.io(IO::NR12);
        //     self.registers.wave1 = bus.io(IO::NR13);
        //     self.registers.wave2 = bus.io(IO::NR14);
        // } else {
        //     self.registers.length = bus.io(IO::NR21);
        //     self.registers.volume = bus.io(IO::NR22);
        //     self.registers.wave1 = bus.io(IO::NR23);
        //     self.registers.wave2 = bus.io(IO::NR24);
        // }
    }
}
