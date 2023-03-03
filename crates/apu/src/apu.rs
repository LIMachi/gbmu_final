use shared::mem::IOBus;
use super::Input;

mod dsg;
mod registers;

use registers::Registers;

#[derive(Default)]
pub struct Apu {
    sample: f64,
    tick: f64,
    input: Input,
    dsg: dsg::DSG,
    registers: Registers,
}

impl Apu {
    pub(crate) fn new(sample_rate: u32, input: Input) -> Self {
        Self {
            sample: 0.,
            tick: 4_194_304. / sample_rate as f64,
            input,
            dsg: dsg::DSG::new(),
            registers: Registers::new()
        }
    }

    pub fn tick(&mut self) {
        self.sample += 1.;
        if self.sample >= self.tick {
            self.input.write_sample(self.dsg.tick(&self.registers));
            self.sample -= self.tick;
        }
    }
}

impl shared::mem::Device for Apu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.dsg.configure(bus);
    }
}
