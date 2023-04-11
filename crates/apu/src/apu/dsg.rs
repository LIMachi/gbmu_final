mod channel;

use std::cell::RefCell;
use std::rc::Rc;
pub(crate) use channel::{Event, Channel};
use shared::io::{AccessMode, IO, IORegs};

#[repr(u8)]
enum Panning {
    Right = 0,
    Left = 4,
}

// impl std::ops::AddAssign<&mut Channel> for DSG {
//     fn add_assign(&mut self, rhs: &mut Channel) {
//         self.output[0] += self.panned(Panning::Left, rhs);
//         self.output[1] += self.panned(Panning::Right, rhs);
//     }
// }

pub(crate) struct DSG {
    output: [f32; 2],
    capacitor: [f32; 2],
    charge_factor: f32,
    cgb_mode: bool
}

impl DSG {
    pub fn new(charge_factor: f32) -> Self {
        Self {
            cgb_mode: Default::default(),
            output: [0.; 2],
            capacitor: [0.; 2],
            charge_factor,
        }
    }

    pub fn set_charge_factor(&mut self, factor: f32) {
        self.charge_factor = factor;
    }

    pub fn hpf(&mut self, volume: u8) -> [f32; 2] {
        let [l, r] = [1 + (volume & 0x70) >> 4, (volume & 0x7) + 1];
        let [vl, vr] = [l as f32 / 8., r as f32 / 8.];
        let [l, r] = self.output; // 0.
        let [cl, cr] = self.capacitor; // -1.
        let [ol, or] = [l - cl, r - cr]; // 1
        self.capacitor = [l - ol * self.charge_factor, r - or * self.charge_factor]; // 0 - 1 * 0.998 -> -0.998
        [ol * vl, or * vr]
    }

    fn panned(&self, side: Panning, channel: &mut Channel, io: &mut IORegs) -> f32 {
        let ctrl = io.io(IO::NR51).value();
        if ctrl & (1 << (side as u8 + channel.channel() as u8)) != 0 {
            channel.output(self.cgb_mode, io)
        } else { 0. }
    }

    pub fn tick(&mut self, channels: &mut [Channel], state: &[Rc<RefCell<bool>>; 4], io: &mut IORegs) -> [f32; 2] {
        self.output = [0.; 2];
        let mut any_dac = false;
        let mut i = 0;
        channels.iter_mut()
            .for_each(|c| {
                if *state[i].as_ref().borrow() {
                    any_dac |= c.dac_enabled(io);
                    // *self += c;
                    self.output[0] += self.panned(Panning::Left, c, io);
                    self.output[1] += self.panned(Panning::Right, c, io);
                }
                i += 1;
        });
        if any_dac { self.hpf(io.io(IO::NR50).value()) } else { [0.; 2] }
    }

    pub fn power_on(&mut self, io: &mut IORegs) {
        io.io(IO::NR51).set_access(IO::NR51.access());
        io.io(IO::NR50).set_access(IO::NR50.access());
    }

    pub fn power_off(&mut self, io: &mut IORegs) {
        io.io(IO::NR51).set_access(AccessMode::rdonly()).direct_write(0);
        io.io(IO::NR50).set_access(AccessMode::rdonly()).direct_write(0);
    }
}
