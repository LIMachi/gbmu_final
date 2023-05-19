use dsg::{Channel, Event};
use shared::audio_settings::AudioSettings;
use shared::io::{IO, IODevice, IORegs};
use shared::mem::IOBus;
use shared::utils::FEdge;

use super::Input;

mod dsg;

pub const TICK_RATE: f64 = 4_194_304.;

pub struct Apu {
    fedge: FEdge,
    div_apu: u8,
    sample: f64,
    sample_rate: u32,
    speed: f64,
    tick: f64,
    input: Input,
    dsg: dsg::DSG,
    channels: Vec<Channel>,
    on: bool,
    cgb: bool,
}

impl Default for Apu {
    fn default() -> Self {
        let sample_rate = 44100;
        Self {
            fedge: Default::default(),
            cgb: false,
            div_apu: 0,
            sample: 0.,
            sample_rate,
            speed: 1.,
            tick: TICK_RATE / sample_rate as f64,
            input: Input::default(),
            dsg: dsg::DSG::new(0.),
            channels: vec![
                Channel::sweep(),
                Channel::pulse(),
                Channel::wave(),
                Channel::noise(),
            ],
            on: false,
        }
    }
}

impl Apu {
    fn charge_factor(&self) -> f32 {
        if self.cgb { 0.998943 } else { 0.999958f32 }
            .powf(TICK_RATE as f32 / self.sample_rate as f32)
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
        self.tick = (TICK_RATE / self.sample_rate as f64) / speed;
        self.sample = 0.;
    }

    pub(crate) fn new(sample_rate: u32, input: Input, cgb: bool) -> Self {
        let channels = vec![
            Channel::sweep(),
            Channel::pulse(),
            Channel::wave(),
            Channel::noise(),
        ];
        let mut apu = Self {
            fedge: FEdge::default(),
            cgb,
            div_apu: 0,
            sample: 0.,
            sample_rate,
            speed: 1.,
            tick: TICK_RATE / sample_rate as f64,
            input,
            dsg: dsg::DSG::new(1.),
            channels,
            on: false,
        };
        apu.dsg.set_charge_factor(apu.charge_factor());
        apu
    }

    pub(crate) fn switch(&mut self, new_rate: u32, input: Input) {
        self.input = input;
        self.sample = 0.;
        self.tick = TICK_RATE / (new_rate as f64) / self.speed;
        self.sample_rate = new_rate;
        self.dsg.set_charge_factor(self.charge_factor());
    }

    fn power(&mut self, io: &mut IORegs, on: bool) {
        if on != self.on {
            if on {
                for channel in self.channels.iter_mut() {
                    channel.power_on(io);
                }
                self.dsg.power_on(io);
            } else {
                for channel in self.channels.iter_mut() {
                    channel.power_off(io);
                }
                self.dsg.power_off(io);
            }
        }
        self.on = on;
    }

    pub fn tick(&mut self, regs: &mut IORegs, ds: bool, settings: &mut AudioSettings) {
        self.sample += 1.;
        if self.sample >= self.tick {
            self.input.write_sample(self.dsg.tick(&mut self.channels, &settings.channels, regs), settings.volume);
            self.sample -= self.tick;
        }
        if !self.on { return; }
        for channel in self.channels.iter_mut() {
            channel.clock(regs);
        }
        if self.fedge.tick(regs.io(IO::DIV).bit(if ds { 5 } else { 4 }) != 0) {
            match self.div_apu {
                0 | 4 => self.channels.iter_mut().for_each(|x| x.event(Event::Length, regs)),
                2 | 6 => self.channels.iter_mut().for_each(|x| {
                    x.event(Event::Sweep, regs);
                    x.event(Event::Length, regs);
                }),
                7 => self.channels.iter_mut().for_each(|x| x.event(Event::Envelope, regs)),
                _ => {}
            }
            self.div_apu += 1;
            self.div_apu %= 8;
        }
    }
}

impl IODevice for Apu {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        match io {
            IO::NR52 => {
                self.power(bus.io_regs(), v & 0x80 != 0)
            }
            IO::NR10 | IO::NR11 | IO::NR12 | IO::NR13 | IO::NR14 => self.channels[0].write(io, v, bus),
            IO::NR21 | IO::NR22 | IO::NR23 | IO::NR24 => self.channels[1].write(io, v, bus),
            IO::NR30 | IO::NR31 | IO::NR32 | IO::NR33 | IO::NR34 => self.channels[2].write(io, v, bus),
            IO::NR41 | IO::NR42 | IO::NR43 | IO::NR44 => self.channels[3].write(io, v, bus),
            _ => {}
        }
    }
}
