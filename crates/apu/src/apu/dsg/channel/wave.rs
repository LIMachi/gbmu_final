use shared::io::{AccessMode, IO, IODevice, IORegs};
use shared::mem::IOBus;
use super::{SoundChannel, Channels};

const PATTERN_REG: [IO; 16] = [
    IO::WaveRam0,
    IO::WaveRam1,
    IO::WaveRam2,
    IO::WaveRam3,
    IO::WaveRam4,
    IO::WaveRam5,
    IO::WaveRam6,
    IO::WaveRam7,
    IO::WaveRam8,
    IO::WaveRam9,
    IO::WaveRamA,
    IO::WaveRamB,
    IO::WaveRamC,
    IO::WaveRamD,
    IO::WaveRamE,
    IO::WaveRamF
];

pub struct Channel {
    cycle: usize,
    freq_timer: u16,
    dac: bool,
    freq: u16,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            freq_timer: 0,
            dac: false,
            freq: 0
        }
    }

    fn frequency(&self, io: &mut IORegs) -> u16 {
        io.io(IO::NR33).value() as u16 | ((io.io(IO::NR34).value() as u16 & 0x7) << 8)
    }

    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn SoundChannel + 'static> {
        Box::new(Self {
            cycle: raw[0] as usize,
            freq_timer: ((raw[1] as u16) << 8) | (raw[2] as u16),
            dac: raw[3] == 1,
            freq: ((raw[4] as u16) << 8) | (raw[5] as u16),
        })
    }
}

impl SoundChannel for Channel {
    fn raw(&self) -> Vec<u8> {
        vec![self.cycle as u8, (self.freq_timer & 0xFF) as u8, ((self.freq_timer >> 8) & 0xFF) as u8, if self.dac { 1 } else { 0 }, (self.freq & 0xFF) as u8, ((self.freq >> 8) & 0xFF) as u8]
    }

    fn output(&self, io: &mut IORegs) -> u8 {
        let v = (io.io(IO::NR32).value() >> 5) & 0x3;
        if v == 0 { return 0 }
        ((io.io(PATTERN_REG[self.cycle / 2]).value() >> ((self.cycle & 1) * 4)) & 0xF) >> (v - 1)
    }

    fn channel(&self) -> Channels { Channels::Wave }

    fn dac_enabled(&self) -> bool {
        self.dac
    }

    fn clock(&mut self, _io: &mut IORegs) {
        if self.freq_timer == 0 {
            self.cycle = (self.cycle + 1) & 0x1F;
            self.freq_timer = 2 * (0x7FF - self.freq);
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self, _io: &mut IORegs) -> bool {
        self.cycle = 1;
        self.freq_timer = 2 * (0x7FF - self.freq) | (self.freq_timer & 0x3);
        false //FIXME: handle special cases/obscure behaviors
    }

    fn length(&self) -> u8 { 0xFF }

    fn on_enable(&mut self, io: &mut IORegs) {
        for pr in PATTERN_REG {
            io.io_mut(pr).set_access(AccessMode::unused());
        }
    }

    fn on_disable(&mut self, io: &mut IORegs) {
        for pr in PATTERN_REG {
            io.io_mut(pr).set_access(AccessMode::rw());
        }
    }

    fn power_on(&mut self, io: &mut IORegs) {
        io.io_mut(IO::NR30).set_access(IO::NR30.access());
        for pr in PATTERN_REG {
            io.io_mut(pr).set_access(AccessMode::rw());
        }
    }

    fn power_off(&mut self, io: &mut IORegs) {
        io.io_mut(IO::NR30).direct_write(0).set_access(AccessMode::rdonly());
        for pr in PATTERN_REG {
            io.io_mut(pr).direct_write(0).set_access(AccessMode::rdonly());
        }
    }
}

impl IODevice for Channel {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        match io {
            IO::NR30 => self.dac = v & 0x80 != 0,
            IO::NR33 | IO::NR34 => self.freq = self.frequency(bus.io_regs()),
            _ => {}
        }
    }
}
