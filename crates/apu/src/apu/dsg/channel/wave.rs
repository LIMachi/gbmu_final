use shared::io::{AccessMode, IO, IOReg, IORegs};
use shared::mem::{Device, IOBus};
use super::{SoundChannel, Channels};

pub struct Channel {
    cycle: usize,
    freq_timer: u16
}

impl Channel {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            freq_timer: 0
        }
    }

    fn frequency(&self, io: &mut IORegs) -> u16 {
        io.io(IO::NR33).value() as u16 | ((io.io(IO::NR34).value() as u16 & 0x7) << 8)
    }
}

impl Device for Channel {
    // fn configure(&mut self, bus: &dyn IOBus) {
    //     self.registers.dac_enable = bus.io(IO::NR30);
    //     self.registers.length = bus.io(IO::NR31);
    //     self.registers.volume = bus.io(IO::NR32);
    //     self.registers.wave1 = bus.io(IO::NR33);
    //     self.registers.wave2 = bus.io(IO::NR34);
    //     self.registers.pattern[0] = bus.io(IO::WaveRam0);
    //     self.registers.pattern[1] = bus.io(IO::WaveRam1);
    //     self.registers.pattern[2] = bus.io(IO::WaveRam2);
    //     self.registers.pattern[3] = bus.io(IO::WaveRam3);
    //     self.registers.pattern[4] = bus.io(IO::WaveRam4);
    //     self.registers.pattern[5] = bus.io(IO::WaveRam5);
    //     self.registers.pattern[6] = bus.io(IO::WaveRam6);
    //     self.registers.pattern[7] = bus.io(IO::WaveRam7);
    //     self.registers.pattern[8] = bus.io(IO::WaveRam8);
    //     self.registers.pattern[9] = bus.io(IO::WaveRam9);
    //     self.registers.pattern[10] = bus.io(IO::WaveRamA);
    //     self.registers.pattern[11] = bus.io(IO::WaveRamB);
    //     self.registers.pattern[12] = bus.io(IO::WaveRamC);
    //     self.registers.pattern[13] = bus.io(IO::WaveRamD);
    //     self.registers.pattern[14] = bus.io(IO::WaveRamE);
    //     self.registers.pattern[15] = bus.io(IO::WaveRamF);
    // }
}

impl SoundChannel for Channel {
    fn output(&self) -> u8 {
        let v = (self.registers.volume.value() >> 5) & 0x3;
        if v == 0 { return 0 }
        ((self.registers.pattern[self.cycle / 2].value() >> ((self.cycle & 1) * 4)) & 0xF) >> (v - 1)
    }

    fn channel(&self) -> Channels { Channels::Wave }

    fn dac_enabled(&self) -> bool {
        self.registers.dac_enable.bit(7) != 0
    }

    fn clock(&mut self) {
        if self.freq_timer == 0 {
            self.cycle = (self.cycle + 1) & 0x1F;
            self.freq_timer = 2 * (0x7FF - self.frequency());
        } else {
            self.freq_timer -= 1;
        }
    }

    fn trigger(&mut self, io: &mut IORegs) -> bool {
        self.cycle = 1;
        self.freq_timer = 2 * (0x7FF - self.frequency()) | (self.freq_timer & 0x3);
        false //FIXME: handle special cases/obscure behaviors
    }

    fn length(&self) -> u8 { 0xFF }

    fn on_enable(&mut self) {
        for w in self.registers.pattern.iter() {
            w.set_access(AccessMode::unused());
        }
    }

    fn on_disable(&mut self) {
        for w in self.registers.pattern.iter() {
            w.set_access(AccessMode::rw());
        }
    }

    fn power_on(&mut self) {
        self.registers.dac_enable.set_access(IO::NR30.access());
        for wave in &self.registers.pattern {
            wave.set_access(IO::WaveRam0.access());
        }
    }

    fn power_off(&mut self) {
        self.registers.dac_enable.set_access(AccessMode::rdonly()).direct_write(0);
        for wave in &self.registers.pattern {
            wave.set_access(AccessMode::rdonly()).direct_write(0);
        }
    }
}
