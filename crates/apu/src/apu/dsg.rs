mod channel;
pub(crate) use channel::{Event, Channel};
use shared::io::{IO, IOReg};

#[repr(u8)]
enum Panning {
    Right = 0,
    Left = 4,
}

impl std::ops::AddAssign<&mut Channel> for DSG {
    fn add_assign(&mut self, rhs: &mut Channel) {
        self.output[0] += self.panned(Panning::Left, rhs);
        self.output[1] += self.panned(Panning::Right, rhs);
    }
}

pub(crate) struct DSG {
    ctrl: IOReg,
    volume: IOReg,
    output: [f32; 2],
    capacitor: [f32; 2],
    tick: usize,
    charge_factor: f32
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
        // TODO add volume knob
        let k = 0.5;
        let vol = self.volume.value();
        let [l, r] = [(vol & 0x70) >> 4 + 1, (vol & 0x7) + 1];
        let [vl, vr] = [l as f32 / 8., r as f32 / 8.];
        let [l, r] = self.output; // 0.
        let [cl, cr] = self.capacitor; // -1.
        let [ol, or] = [l - cl, r - cr]; // 1
        self.capacitor = [l - ol * self.charge_factor, r - or * self.charge_factor]; // 0 - 1 * 0.998 -> -0.998
        [ol * vl * k, or * vr * k]
    }

    fn panned(&self, side: Panning, channel: &mut Channel) -> f32 {
        if self.ctrl.value() & (1 << (side as u8 + channel.channel() as u8)) != 0 {
            channel.output()
        } else { 0. }
    }

    pub fn tick(&mut self, channels: &mut [Channel]) -> [f32; 2] {
        self.output = [0.; 2];
        let mut any_dac = false;
        channels.iter_mut()
            .for_each(|c| {
            any_dac |= c.dac_enabled();
            *self += c;
        });
        self.tick += 1;
        if any_dac { self.hpf() } else { [0.; 2] }
    }
}

impl shared::mem::Device for DSG {
    fn configure(&mut self, bus: &dyn shared::mem::IOBus) {
        self.ctrl = bus.io(IO::NR51);
        self.volume = bus.io(IO::NR50);
    }
}
