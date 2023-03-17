use std::borrow::BorrowMut;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use lcd::{Lcd, LCD};
use mem::{oam::{Oam, Sprite}, Vram};
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
use shared::utils::Cell;

const TILEDATA_WIDTH: usize = 16 * 16;
const TILEDATA_HEIGHT: usize = 24 * 16;

pub(crate) struct REdge {
    inner: bool
}

impl REdge {
    fn new() -> Self {
        Self { inner: false }
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
    pub y: u8
}

#[derive(Default)]
pub(crate) struct Scroll {
    pub x: u8,
    pub y: u8
}

pub(crate) struct Ppu {
    pub(crate) dots: usize,
    pub(crate) oam: Rc<RefCell<Lock<Oam>>>,
    pub(crate) vram: Rc<RefCell<Lock<Vram>>>,
    pub(crate) state: Box<dyn State>,
    pub(crate) regs: Registers,
    pub(crate) cram: cram::CRAM,
    pub(crate) sprites: Vec<usize>,
    pub(crate) lcd: Lcd,
    pub(crate) win: Window,
    pub(crate) sc: Scroll,
    pub(crate) stat: REdge,
    pub(crate) lcdc: LCDC,
    pub(crate) tiledata: Vec<Image<TILEDATA_WIDTH, TILEDATA_HEIGHT>>
}

impl Ppu {
    pub fn new(lcd: Lcd) -> Self {
        use shared::mem::Locked;
        let sprites = Vec::with_capacity(10);
        Self {
            sc: Scroll::default(),
            dots: 0,
            oam: Oam::new().locked().cell(),
            vram: Vram::new().locked().cell(),
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

    pub(crate) fn oam(&self) -> Ref<Lock<Oam>> { self.oam.as_ref().borrow() }
    pub(crate) fn vram(&self) -> Ref<Lock<Vram>> { self.vram.as_ref().borrow() }
    pub(crate) fn oam_mut(&self) -> RefMut<Lock<Oam>> { self.oam.as_ref().borrow_mut() }
    pub(crate) fn vram_mut(&self) -> RefMut<Lock<Vram>> { self.vram.as_ref().borrow_mut() }

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

        match mode {
            Mode::Search => self.oam_mut().lock(Source::Ppu),
            Mode::Transfer => self.vram_mut().lock(Source::Ppu),
            Mode::HBlank => {
                self.oam_mut().unlock(Source::Ppu);
                self.vram_mut().unlock(Source::Ppu);
            }
            _ => {}
        };
    }

    pub(crate) fn tick(&mut self) {
        self.cram.tick();
        let lcdc = LCDC(self.regs.lcdc.read());
        if self.lcdc.enabled() && !lcdc.enabled() {
            self.dots = 0;
            self.regs.ly.direct_write(0);
            self.set_state(Box::new(HState::last()));
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
        self.oam().inner().sprites[index]
    }

    pub fn set(&mut self, lx: usize, ly: usize, pixel: Pixel) {
        self.lcd.set(lx, ly, self.cram.color(pixel));
    }
}

impl Device for Ppu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.regs.configure(bus);
        self.cram.configure(bus);
        self.vram_mut().configure(bus);
    }
}
