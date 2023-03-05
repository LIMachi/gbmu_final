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

pub struct Channel {
    envelope: Envelope,
    sweep: bool,
    shadow: u16,
    registers: Registers,
    sweep_negate: bool
}

impl Channel {
    pub fn new(sweep: bool) -> Self {
        Self {
            envelope: Envelope::default(),
            sweep,
            sweep_negate: false,
            shadow: 0,
            registers: Registers::default(),
        }
    }

    fn frequency(&self) -> u16 {
        self.registers.wave1.read() as u16 | ((self.registers.wave2.read() as u16 & 0x7) << 8)
    }

    fn update_sweep(&mut self) {
        if self.registers.sweep.dirty() {
            self.registers.sweep.reset_dirty();
            self.sweep_negate = self.registers.sweep.bit(3) == 0;
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
    fn output(&self) -> f32 {
        todo!()
    }

    fn channel(&self) -> Channels {
        if self.sweep { Channels::Sweep } else { Channels::Pulse }
    }

    fn dac_enabled(&self) -> bool {
        self.registers.volume.value() & 0xF8 != 0
    }

    fn clock(&mut self) {
        self.update_sweep();
        self.update_envelope();
    }

    fn trigger(&mut self) {
        self.envelope.trigger();
    }

    fn sweep(&mut self) -> bool {
        if self.sweep {
            false
        }
        else {
            false
        }
    }

    fn envelope(&mut self) {
        self.envelope.clock();
    }

    fn length(&self) -> u8 {
        todo!()
    }
}

impl Device for Channel {
    fn configure(&mut self, bus: &dyn IOBus) {
        if self.sweep {
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
