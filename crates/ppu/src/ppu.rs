use std::collections::HashSet;
use lcd::{Lcd, LCD};
use mem::oam::Sprite;
use mem::{Lock, Oam, Vram};
use shared::io::LCDC;
use shared::mem::{Device, IOBus, Mem, *};
use shared::utils::image::Image;
use crate::ppu::registers::Registers;

mod fetcher;
mod cram;
mod pixel;
mod fifo;
mod registers;
mod states;

use states::*;
use pixel::Pixel;

const TILEDATA_WIDTH: usize = 16 * 16;
const TILEDATA_HEIGHT: usize = 24 * 16;

pub(crate) struct REdge {
    inner: bool,
    input: bool
}

impl REdge {
    fn new() -> Self {
        Self { inner: false, input: false }
    }

    fn tick(&mut self, input: bool) -> bool {
        let res = input && !self.inner;
        self.inner = input;
        res
    }
}

#[derive(Default)]
pub(crate) struct Window {
    pub scan_enabled: bool,
    pub enabled: bool,
    pub x: u8,
    pub y: u8
}

#[derive(Default)]
pub(crate) struct Scroll {
    pub x: u8,
    pub y: u8
}

pub(crate) struct Ppu {
    pub(crate) tile_cache: HashSet<usize>,
    pub(crate) dots: usize,
    pub(crate) oam: Lock<Oam>,
    pub(crate) vram: Lock<Vram>,
    pub(crate) state: Box<dyn State>,
    pub(crate) regs: Registers,
    pub(crate) cram: cram::CRAM,
    pub(crate) sprites: Vec<usize>,
    pub(crate) lcd: Lcd,
    pub(crate) win: Window,
    pub(crate) sc: Scroll,
    pub(crate) stat: REdge,
    pub(crate) tiledata: Vec<Image<TILEDATA_WIDTH, TILEDATA_HEIGHT>>,
    pub(crate) lcdc: LCDC,
    locked: bool
}

impl Ppu {
    pub fn new(lcd: Lcd) -> Self {
        use mem::lock::Locked;
        let sprites = Vec::with_capacity(10);
        Self {
            locked: false,
            sc: Scroll::default(),
            tile_cache: HashSet::with_capacity(384),
            dots: 0,
            oam: Oam::new().lock(),
            vram: Vram::new().lock(),
            regs: Registers::default(),
            cram: cram::CRAM::default(),
            state: Box::new(VState::new()),
            sprites,
            lcd,
            lcdc: LCDC(0),
            win: Default::default(),
            tiledata: vec![],
            stat: REdge::new()
        }
    }

    fn set_state(&mut self, state: Box<dyn State>) {
        let mode = state.mode();
        self.state = state;
        self.regs.stat.reset(0);
        self.regs.stat.reset(1);
        if (mode as u8 & 0x1) != 0 { self.regs.stat.set(0); }
        if (mode as u8 & 0x2) != 0 { self.regs.stat.set(1); }
        let input = self.regs.stat.bit(3) != 0 && mode == Mode::HBlank;
        let input = input || (self.regs.stat.bit(4) != 0 && mode == Mode::VBlank);
        let input = input || (self.regs.stat.bit(5) != 0 && mode == Mode::Search);
        let input = input || (self.regs.stat.bit(6) != 0 && self.regs.stat.bit(2) != 0);
        if self.stat.tick(input) {
            self.regs.interrupt.set(1);
        }
    }

    pub(crate) fn tick(&mut self) {
        self.cram.tick();
        let lcdc = LCDC(self.regs.lcdc.read());
        if self.lcdc.enabled() && !lcdc.enabled() {
            self.dots = 0;
            self.regs.ly.direct_write(0);
            self.state = Box::new(HState::last());
            self.lcd.disable();
        }
        self.lcdc = lcdc;
        self.dots += 1;
        if self.lcdc.enabled() {
            if self.regs.ly.read() == self.regs.lyc.read() { self.regs.stat.set(2); } else { self.regs.stat.reset(2); }
            let mut state = std::mem::replace(&mut self.state, OamState::new().boxed());
            if let Some(next) = state.tick(self) {
                let mode = next.mode();
                if mode == Mode::VBlank {
                    self.dots = 0;
                    self.lcd.enable();
                }
                self.set_state(next);
            } else { self.state = state; };
        } else if self.dots == 65664 {
            self.set_state(VState::immediate().boxed());
        }
    }

    pub fn sprite(&self, index: usize) -> Sprite {
        self.oam.inner().sprites[index]
    }

    pub fn set(&mut self, lx: usize, ly: usize, pixel: Pixel) {
        self.lcd.set(lx, ly, self.cram.color(pixel));
    }
}

impl Device for Ppu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.regs.configure(bus);
        self.cram.configure(bus);
        self.vram.configure(bus);
    }
}

const TILE_DATA_END: u16 = 0x97FF;
const TILEMAP: u16 = 0x9800;

impl Mem for Ppu {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            OAM..=OAM_END => self.oam.read(addr, absolute),
            VRAM..=VRAM_END => self.vram.read(addr, absolute),
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        // TODO prevent write access if oam / vram locked
        match absolute {
            OAM..=OAM_END => self.oam.write(addr, value, absolute),
            VRAM..=TILE_DATA_END => {
                self.tile_cache.insert(addr as usize / 16);
                self.vram.write(addr, value, absolute)
            }
            VRAM..=VRAM_END => {
                self.vram.write(addr, value, absolute)
            },
            _ => unreachable!()
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        match st {
            OAM => self.oam.get_range(st, len),
            VRAM => self.vram.get_range(st, len),
            _ => unimplemented!()
        }
    }
}
