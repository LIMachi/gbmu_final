use apu::Apu;
use shared::audio_settings::AudioSettings;
use shared::breakpoints::Breakpoints;
use shared::input::Keybindings;
use crate::Timer;

#[derive(Default)]
pub struct ConsoleBuilder {
    apu: Option<Apu>,
    serial: Option<serial::Port>,
    joypad: Option<joy::Joypad>,
    skip: bool,
    cgb: bool
}

impl ConsoleBuilder {

    pub fn with_link(mut self, cable: serial::com::Serial) -> Self {
        self.serial = Some(serial::Port::new(cable)); self
    }

    pub fn with_apu(mut self, apu: Apu) -> Self {
        self.apu = Some(apu); self
    }

    pub fn with_keybinds(mut self, keybinds: Keybindings) -> Self {
        self.joypad = Some(joy::Joypad::new(keybinds)); self
    }

    pub fn skip_boot(mut self, skip: bool) -> Self {
        self.skip = skip; self
    }

    pub fn set_cgb(mut self, cgb: bool) -> Self {
        self.cgb = cgb; self
    }

    pub fn build(self) -> Devices {
        let mut cpu = cpu::Cpu::default();

        let lcd = lcd::Lcd::default();
        let ppu = ppu::Controller::new();
        if self.skip { cpu.skip_boot(self.cgb); }

        Devices {
            cpu: cpu::Cpu::default(),
            ppu,
            lcd,
            apu: self.apu.unwrap_or_default(),
            dma: ppu::Dma::default(),
            hdma: ppu::Hdma::default(),
            serial: self.serial.unwrap_or_default(),
            timer: Timer::default(),
            joy: self.joypad.unwrap_or_default(),
        }
    }
}

pub struct Devices {
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Controller,
    pub joy: joy::Joypad,
    pub lcd: lcd::Lcd,
    pub dma: ppu::Dma,
    pub hdma: ppu::Hdma,
    pub timer: Timer,
    pub apu: Apu,
    pub serial: serial::Port,
}

impl Devices {
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }
}

pub struct Settings<'a> {
    pub breakpoints: &'a mut Breakpoints,
    pub sound: &'a mut AudioSettings,
}
