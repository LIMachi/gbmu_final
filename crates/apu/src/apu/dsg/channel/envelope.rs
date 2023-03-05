
#[derive(Default)]
pub struct Envelope {
    base: u8,
    value: u8,
    timer: u8,
    period: u8,
    increase: bool
}

impl Envelope {
    pub fn update(&mut self, data: u8) {
        self.value = data >> 4;
        self.base = self.value;
        self.period = data & 0x7;
        self.increase = data & 0x8 != 0;
        if self.period == 0 { self.period = 8; }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 { return };
        self.timer -= 1;
        if self.timer == 0 {
            let t = if self.increase { self.value + 1 } else { self.value.wrapping_sub(1) };
            if t <= 0xF {
                self.value = t;
                self.timer = self.period;
            }
        }
    }

    pub fn volume(&self) -> u8 {
        self.value
    }

    pub fn trigger(&mut self) {
        self.timer = self.period;
        self.value = self.base;
    }
}
