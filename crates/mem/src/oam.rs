use serde::{Deserialize, Serialize};
use shared::mem::Mem;
use shared::utils::serde_arrays;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8
}

impl Sprite {
    pub fn screen_x(&self) -> u8 { self.x.wrapping_sub(8) }
    pub fn screen_y(&self) -> u8 { self.y.wrapping_sub(16) }

    pub fn unavailable() -> Self {
        Self { x: 0xFF, y: 0xFF, tile: 0xFF, flags: 0xFF }
    }
}

impl Mem for Sprite {
    fn read(&self, addr: u16, _absolute: u16) -> u8 {
        match addr {
            0 => self.y,
            1 => self.x,
            2 => self.tile,
            3 => self.flags,
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, _absolute: u16) {
        match addr {
            0 => self.y = value,
            1 => self.x = value,
            2 => self.tile = value,
            3 => self.flags = value,
            _ => unreachable!()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oam {
    #[serde(with = "serde_arrays")]
    pub sprites: [Sprite; 40]
}

impl Mem for Oam {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match addr {
            0..=159 => self.sprites[(addr / 4) as usize].read(addr % 4, absolute),
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match addr {
            0..=159 => self.sprites[(addr / 4) as usize].write(addr % 4, value, absolute),
            _ => unreachable!()
        }
    }

    fn get_range(&self, _st: u16, _len: u16) -> Vec<u8> {
        self.sprites.as_ref().iter().map(|x| [x.x, x.y, x.tile, x.flags])
            .flatten()
            .collect()
    }
}

impl Oam {
    pub fn new() -> Self {
        Self { sprites: [Sprite::default(); 40] }
    }
}
