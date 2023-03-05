mod channel;
pub(crate) use channel::{Event, SoundChannel, Channel};
use shared::io::{IO, IOReg};
use crate::apu::dsg::channel::Channels;

#[repr(u8)]
enum Panning {
    Right = 0,
    Left = 4,
}

impl std::ops::AddAssign<&Channel> for DSG {
    fn add_assign(&mut self, rhs: &Channel) {
        self.panned(Panning::Left, rhs);
        self.panned(Panning::Right, rhs);
    }
}

#[derive(Default)]
pub(crate) struct DSG {
    ctrl: IOReg,
    volume: IOReg,
    output: [f32; 2],
    capacitor: [f32; 2],
    tick: usize,
    charge_factor: f32,
}

impl DSG {
    pub fn new(charge_factor: f32) -> Self {
        Self {
            ctrl: Default::default(),
            volume: Default::default(),
            output: [0.; 2],
            capacitor: [0.; 2],
            tick: 0,
            charge_factor
        }
    }

    pub fn set_charge_factor(&mut self, factor: f32) {
        self.charge_factor = factor;
    }

    pub fn hpf(&mut self) -> [f32; 2] {
        let [l, r] = self.output;
        let [cl, cr] = self.capacitor;
        let [ol, or] = [l - cl, r - cr];
        self.capacitor = [l - ol * self.charge_factor, r - or * self.charge_factor];
        [ol, or]
    }

    fn panned(&self, side: Panning, channel: &Channel) -> f32 {
        if self.ctrl.value() & (1 << (side as u8 + channel.channel() as u8)) != 0 {
            channel.output()
        } else { 0. }
    }

    pub fn tick(&mut self, channels: &mut [Channel]) -> [f32; 2] {
        self.output = [0.; 2];
        let mut any_dac = false;
        channels.iter().for_each(|c| {
            any_dac |= c.dac_enabled();
            *self += c;
        });
        self.tick += 1;
        let a = (2. * std::f32::consts::PI * 220. * self.tick as f32 / 44800.).sin();
        if any_dac { self.hpf() } else { [0.; 2] }
    }
}

impl shared::mem::Device for DSG {
    fn configure(&mut self, bus: &dyn shared::mem::IOBus) {
        self.ctrl = bus.io(IO::NR51);
        self.volume = bus.io(IO::NR50);
    }
}
