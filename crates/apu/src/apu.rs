use shared::mem::IOBus;
use super::Input;

mod dsg;

use shared::io::{IO, IOReg};
use dsg::{Channel, Event, SoundChannel};
use shared::utils::FEdge;

pub struct Apu {
    fedge: FEdge,
    div_apu: u8,
    sample: f64,
    sample_rate: u32,
    tick: f64,
    input: Input,
    dsg: dsg::DSG,
    sound: IOReg,
    div: IOReg,
    ds: IOReg,
    channels: Vec<Channel>
}

impl Default for Apu {
    fn default() -> Self {
        let sample_rate = 44100;
        Self {
            fedge: Default::default(),
            div_apu: 0,
            sample: 0.,
            sample_rate,
            tick: 4_194_304. / sample_rate as f64,
            input: Input::default(),
            dsg: dsg::DSG::new(0.),
            channels: vec![],
            sound: Default::default(),
            div: Default::default(),
            ds: Default::default()
        }
    }
}

impl Apu {
    fn charge_factor(&self) -> f32 {
        // TODO match on hypothetical cgb bus (0.998943 for cgb)
        0.999958f32.powf(4194304./ self.sample_rate as f32)
    }

    pub(crate) fn new(sample_rate: u32, input: Input) -> Self {
        let channels = vec![
            Channel::sweep(),
            Channel::pulse(),
            Channel::wave(),
            // Channel::noise(),
        ];
        Self {
            fedge: FEdge::default(),
            div_apu: 0,
            sample: 0.,
            sample_rate,
            tick: 4_194_304. / sample_rate as f64,
            input,
            sound: IOReg::unset(),
            dsg: dsg::DSG::new(1.),
            channels,
            div: IOReg::unset(),
            ds: IOReg::rdonly(),
        }
    }

    // TODO call this on Events::AudioDevice
    pub fn update_sample_rate(&mut self, sample_rate: u32) {
        self.sample = 0.;
        self.tick = 4_194_304. / sample_rate as f64;
        self.sample_rate = sample_rate;
        self.dsg.set_charge_factor(self.charge_factor());
    }

    pub fn tick(&mut self) {
        self.sample += 1.;
        if self.sample >= self.tick {
            self.input.write_sample(self.dsg.tick(&mut self.channels));
            self.sample -= self.tick;
        }
        for channel in self.channels.iter_mut() {
            channel.clock();
        }
        self.dsg.clock();
        if self.fedge.tick(self.div.bit(if self.ds.bit(7) != 0 { 5 } else { 4 }) != 0) {
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
        self.ds = bus.io(IO::KEY1);
    }
}
