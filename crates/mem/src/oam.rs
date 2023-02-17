use shared::mem::Mem;

#[derive(Default, Copy, Clone)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8
}

impl Mem for Sprite {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match addr {
            0 => self.y,
            1 => self.x,
            2 => self.tile,
            3 => self.flags,
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match addr {
            0 => self.y = value,
            1 => self.x = value,
            2 => self.tile = value,
            3 => self.flags = value,
            _ => unreachable!()
        }
    }
}

pub struct Oam {
    pub sprites: [Sprite; 40]
}

impl Mem for Oam {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match addr {
            0..160 => self.sprites[(addr / 4) as usize].read(addr % 4, absolute),
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match addr {
            0..160 => self.sprites[(addr / 4) as usize].write(addr % 4, value, absolute),
            _ => unreachable!()
        }
    }
}

impl Oam {
    pub fn new() -> Self {
        Self { sprites: [Sprite::default(); 40] }
    }
}
