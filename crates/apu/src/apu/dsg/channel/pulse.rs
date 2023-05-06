use shared::io::{AccessMode, IO, IODevice, IORegs};
use shared::mem::{IOBus};
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
    pub fn raw(&self) -> Vec<u8> {
        vec![(self.shadow & 0xFF) as u8, ((self.shadow >> 8) & 0xFF) as u8, self.pace, self.timer, if self.negate { 1 } else { 0 }, self.shift, if self.ran { 1 } else { 0 }, if self.enabled { 1 } else { 0 }]
    }

    pub fn from_raw(raw: &[u8]) -> Self {
        let shadow: u16 = raw[0] as u16 | ((raw[1] as u16) << 8);
        Self {
            shadow,
            pace: raw[2],
            timer: raw[3],
            negate: raw[4] == 1,
            shift: raw[5],
            ran: raw[6] == 1,
            enabled: raw[7] == 1
        }
    }

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
                    io.io_mut(wave1).direct_write(f as u8);
                    let wave2 = io.io_mut(wave2);
                    wave2.direct_write((wave2.value() & 0x40) | ((f >> 8) & 0x7) as u8);
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
    freq: u16,
    envelope: Envelope,
    sweep: Sweep,
    has_sweep: bool,
    registers: Registers,
    triggered: bool,
    dac: bool
}

impl Channel {
    pub fn new(sweep: bool) -> Self {
        Self {
            triggered: false,
            dac: false,
            freq: 0,
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

    pub(crate) fn from_raw(sweep: bool, raw: Vec<u8>) -> Box<dyn SoundChannel + 'static> {
        let mut out = Self::new(sweep);
        out.envelope = Envelope::from_raw(&raw[..5]);
        out.triggered = raw[5] == 1;
        out.dac = raw[6] == 1;
        out.freq = raw[7] as u16 | ((raw[8] as u16) << 8);
        out.freq_timer = raw[9] as u16 | ((raw[10] as u16) << 8);
        out.cycle = raw[11] as usize;
        if sweep {
            out.sweep = Sweep::from_raw(&raw[12..]);
        }
        Box::new(out)
    }

    fn frequency(&self, io: &mut IORegs) -> u16 {
        io.io(self.registers.wave1).value() as u16 | ((io.io(self.registers.wave2).value() as u16 & 0x7) << 8)
    }
}

impl SoundChannel for Channel {
    fn raw(&self) -> Vec<u8> {
        let mut out = self.envelope.raw();
        out.push(if self.triggered { 1 } else { 0 });
        out.push(if self.dac { 1 } else { 0 });
        out.push((self.freq & 0xFF) as u8);
        out.push(((self.freq >> 8) & 0xFF) as u8);
        out.push((self.freq_timer & 0xFF) as u8);
        out.push(((self.freq_timer >> 8) & 0xFF) as u8);
        out.extend(self.cycle.to_le_bytes());
        if self.has_sweep {
            out.extend(self.sweep.raw())
        }
        out
    }

    fn output(&self, io: &mut IORegs) -> u8 {
        let waveform = io.io(self.registers.length).value() >> 6;
        DUTY_CYCLES[waveform as usize][self.cycle] * self.envelope.volume()
    }

    fn channel(&self) -> Channels {
        if self.has_sweep { Channels::Sweep } else { Channels::Pulse }
    }

    fn dac_enabled(&self) -> bool { self.dac }

    fn clock(&mut self, _io: &mut IORegs) {
        if !self.triggered { return }
        if self.freq_timer == 0 {
            self.cycle = (self.cycle + 1) & 0x7;
            self.freq_timer = 4 * (0x7FF - self.freq);
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self, _io: &mut IORegs) -> bool {
        self.triggered = true;
        self.envelope.trigger();
        self.freq_timer = 4 * (0x7FF - self.freq) | (self.freq_timer & 0x3);
        self.sweep.trigger(self.freq) > 2047
    }

    fn sweep(&mut self, io: &mut IORegs) -> bool {
        if self.has_sweep {
            let overflow = self.sweep.tick(io, self.registers.wave1, self.registers.wave2);
            self.freq = self.frequency(io);
            overflow
        }
        else { false }
    }

    fn envelope(&mut self) {
        self.envelope.clock();
    }

    fn length(&self) -> u8 { 0x3F }

    fn power_on(&mut self, io: &mut IORegs) {
        if self.has_sweep {
            io.io_mut(IO::NR10).set_access(IO::NR10.access());
        }
    }

    fn power_off(&mut self, io: &mut IORegs) {
        if self.has_sweep {
            io.io_mut(IO::NR10).direct_write(0).set_access(AccessMode::rdonly());
        }
    }
}

impl IODevice for Channel {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        if io == self.registers.volume {
            self.envelope.update(v);
            self.dac = v & 0xF8 != 0;
        }
        else if io == self.registers.wave1 || io == self.registers.wave2 {
            self.freq = self.frequency(bus.io_regs());
        } else if self.has_sweep && io == IO::NR10 { self.sweep.update(v); }
    }
}
