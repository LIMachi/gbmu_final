use serde::{Deserialize, Serialize};
use apu::{Apu, Controller};
use joy::Joypad;
use shared::audio_settings::AudioSettings;
use shared::breakpoints::Breakpoints;
use shared::io::{IO, IODevice};
use shared::mem::IOBus;

use crate::Timer;

#[derive(Default)]
pub struct ConsoleBuilder<'a> {
    apu: Option<&'a Controller>,
    serial: Option<serial::Port>,
    skip: bool,
    cgb: bool,
}

impl<'a> ConsoleBuilder<'a> {
    pub fn with_link(mut self, cable: serial::com::Serial) -> Self {
        self.serial = Some(serial::Port::new(cable));
        self
    }

    pub fn with_sound_driver(mut self, apu: &'a Controller) -> Self {
        self.apu = Some(apu);
        self
    }

    // pub fn with_keybinds(mut self) -> Self {
    //     self.joypad = Some(joy::Joypad::new()); self
    // }

    pub fn skip_boot(mut self, skip: bool) -> Self {
        self.skip = skip;
        self
    }

    pub fn set_cgb(mut self, cgb: bool) -> Self {
        self.cgb = cgb;
        self
    }

    pub fn build(self) -> Devices {
        let mut cpu = cpu::Cpu::default();

        let lcd = lcd::Lcd::default();
        let ppu = ppu::Controller::new();
        let joy = Joypad::new();
        if self.skip { cpu.skip_boot(self.cgb); }

        Devices {
            cpu,
            ppu,
            lcd,
            apu: self.apu.map(|x| x.apu(self.cgb)).unwrap_or_default(),
            dma: ppu::Dma::default(),
            hdma: ppu::Hdma::default(),
            serial: self.serial.unwrap_or_default(),
            timer: Timer::default(),
            joy,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Devices {
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Controller,
    pub joy: Joypad,
    pub lcd: lcd::Lcd,
    pub dma: ppu::Dma,
    pub hdma: ppu::Hdma,
    pub timer: Timer,
    pub apu: Apu,
    #[serde(default, skip)]
    pub serial: serial::Port,
}

impl Default for Devices {
    fn default() -> Self { Devices::builder().build() }
}

impl Devices {
    pub fn builder<'a>() -> ConsoleBuilder<'a> {
        ConsoleBuilder::default()
    }

    pub fn io_write(&mut self, io: u16, v: u8, bus: &mut dyn IOBus) {
        if let Ok(io) = IO::try_from(io) {
            self.ppu.write(io, v, bus);
            self.dma.write(io, v, bus);
            self.hdma.write(io, v, bus);
            self.apu.write(io, v, bus);
            self.serial.write(io, v, bus);
            self.joy.write(io, v, bus);
        }
    }
}

pub struct Settings<'a> {
    pub breakpoints: &'a mut Breakpoints,
    pub sound: &'a mut AudioSettings,
}
//
// #[derive(Serialize, Deserialize)]
// struct DevicesState {
//     cpu: <cpu::Cpu as shared::emulator::State>::Storage,
// }
//
// impl shared::emulator::State for Devices {
//     type Storage = DevicesState;
//
//     fn load_state(data: shared::emulator::Storage, ctx: &mut impl Emulator) -> Self {
//         Self {
//             cpu: Cpu::load_state(data.cpu, ctx)
//         }
//     }
//
//     fn save_state(&self) -> shared::emulator::Storage {
//         Self {
//             cpu: self.cpu.save_state()
//         }
//     }
// }
