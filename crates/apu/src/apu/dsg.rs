use std::f32::consts::PI;
use super::Registers;

#[derive(Default)]
pub(crate) struct DSG {
    tick: usize,
}

impl DSG {
    pub fn new() -> Self {
        Self {
            tick: 0
        }
    }

    pub fn tick(&mut self, registers: super) -> [f32; 2] {
        self.tick += 1;
        let a = (2. * PI * 440. * self.tick as f32 / sample_rate as f32).sin();
        [a; 2]
    }
}

impl shared::mem::Device for DSG {
    fn configure(&mut self, bus: &dyn shared::mem::IOBus) {

    }
}
