use shared::mem::IOBus;
use super::Input;

mod dsg;

use shared::io::{IO, IOReg, IORegs};
use dsg::{Channel, Event};
use shared::audio_settings::AudioSettings;
use shared::utils::FEdge;

pub const TICK_RATE: f64 = 4_194_304.;

pub struct Apu {
    fedge: FEdge,
    div_apu: u8,
    sample: f64,
    sample_rate: u32,
    tick: f64,
    input: Input,
    dsg: dsg::DSG,
    channels: Vec<Channel>,
    on: bool,
    settings: AudioSettings
}

impl Default for Apu {
    fn default() -> Self {
        let sample_rate = 44100;
        Self {
            fedge: Default::default(),
            div_apu: 0,
            sample: 0.,
            sample_rate,
            tick: TICK_RATE / sample_rate as f64,
            input: Input::default(),
            dsg: dsg::DSG::new(0.),
            channels: vec![],
            on: false,
            settings: Default::default(),
        }
    }
}

impl Apu {
    fn charge_factor(&self) -> f32 {
        // TODO match on hypothetical cgb bus (0.998943 for cgb)
        0.999958f32.powf(TICK_RATE as f32 / self.sample_rate as f32)
    }

    pub(crate) fn new(sample_rate: u32, input: Input, settings: AudioSettings) -> Self {
        #[cfg(feature = "debug")]
        let channels = vec![];
        #[cfg(not(feature = "debug"))]
        let channels = vec![
            Channel::sweep(),
            Channel::pulse(),
            Channel::wave(),
            Channel::noise(),
        ];
        Self {
            fedge: FEdge::default(),
            div_apu: 0,
            sample: 0.,
            sample_rate,
            tick: TICK_RATE / sample_rate as f64,
            input,
            dsg: dsg::DSG::new(1.),
            channels,
            on: false,
            settings
        }
    }

    // TODO call this on Events::AudioDevice
    pub fn update_sample_rate(&mut self, sample_rate: u32) {
        self.sample = 0.;
        self.tick = TICK_RATE / sample_rate as f64;
        self.sample_rate = sample_rate;
        self.dsg.set_charge_factor(self.charge_factor());
    }

    fn power(&mut self, io: &mut IORegs, on: bool) {
        self.sound.reset_dirty();
        if on != self.on {
            if on {
                log::info!("APU on");
                for channel in self.channels.iter_mut() {
                    channel.power_on(io);
                }
                self.dsg.power_on(io);
            } else {
                log::info!("APU off");
                for channel in self.channels.iter_mut() {
                    channel.power_off(io);
                }
                self.dsg.power_off(io);
            }
        }
        self.on = on;
    }

    pub fn tick(&mut self, regs: &mut IORegs, ds: bool) {
        self.sample += 1.;
        if self.sample >= self.tick {
            self.input.write_sample(self.dsg.tick(&mut self.channels, &self.settings.channels), *self.settings.volume.as_ref().borrow());
            self.sample -= self.tick;
        }
        let sound = regs.io(IO::NR52);
        if sound.dirty() { sound.reset_dirty(); let on = sound.bit(7) != 0; self.power(regs, on); }
        if !self.on { return; }
        for channel in self.channels.iter_mut() {
            channel.clock(regs);
        }
        if self.fedge.tick(self.div.bit(if ds { 5 } else { 4 }) != 0) {
            match self.div_apu {
                0 | 4 => self.channels.iter_mut().for_each(|x| x.event(Event::Length)),
                2 | 6 => self.channels.iter_mut().for_each(|x| {
                    x.event(Event::Sweep);
                    x.event(Event::Length);
                }),
                7 => self.channels.iter_mut().for_each(|x| x.event(Event::Envelope)),
                _ => {}
            }
            self.div_apu += 1;
            self.div_apu %= 8;
        }
    }
}

impl shared::mem::Device for Apu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.channels.iter_mut().for_each(|x| {
            x.configure(bus);
        });
        self.sound = bus.io(IO::NR52);
        self.dsg.configure(bus);
        self.dsg.set_charge_factor(self.charge_factor());
        self.div = bus.io(IO::DIV);
    }
}
