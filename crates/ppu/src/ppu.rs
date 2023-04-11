use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use lcd::{Lcd, LCD};
use mem::{oam::{Oam, Sprite}, Vram};
use shared::io::{IO, IORegs, LCDC};
use shared::mem::{Device, IOBus, *};
use crate::ppu::registers::Registers;

mod fetcher;
mod cram;
mod pixel;
mod fifo;
mod registers;
mod states;

pub(crate) type PpuState = Box<dyn State>;

use states::*;
use pixel::Pixel;
use shared::utils::Cell;

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
    pub(crate) cram: cram::CRAM,
    pub(crate) sprites: Vec<usize>,
    pub(crate) win: Window,
    pub(crate) sc: Scroll,
    pub(crate) stat: REdge,
    pub(crate) lcdc: LCDC
}

impl Ppu {
    pub fn new() -> Self {
        let sprites = Vec::with_capacity(10);
        Self {
            sc: Scroll::default(),
            dots: 0,
            oam: Oam::new().locked().cell(),
            vram: Vram::new().locked().cell(),
            cram: cram::CRAM::default(),
            sprites,
            lcdc: LCDC(0),
            win: Default::default(),
            stat: REdge::new()
        }
    }

    pub(crate) fn oam(&self) -> Ref<Lock<Oam>> { self.oam.as_ref().borrow() }
    pub(crate) fn vram(&self) -> Ref<Lock<Vram>> { self.vram.as_ref().borrow() }
    pub(crate) fn oam_mut(&self) -> RefMut<Lock<Oam>> { self.oam.as_ref().borrow_mut() }
    pub(crate) fn vram_mut(&self) -> RefMut<Lock<Vram>> { self.vram.as_ref().borrow_mut() }

    fn set_state(&mut self, regs: &mut IORegs, state: &mut Box<dyn State>, next: Box<dyn State>) {
        let mode = next.mode();
        let stat = regs.io(IO::STAT);
        *state = next;
        stat.reset(0);
        stat.reset(1);
        if (mode as u8 & 0x1) != 0 { stat.set(0); }
        if (mode as u8 & 0x2) != 0 { stat.set(1); }
        let input = stat.bit(3) != 0 && mode == Mode::HBlank;
        let input = input || (stat.bit(4) != 0 && mode == Mode::VBlank);
        let input = input || (stat.bit(5) != 0 && mode == Mode::Search);
        let input = input || (stat.bit(6) != 0 && stat.bit(2) != 0);
        if self.stat.tick(input) {
            regs.io(IO::IF).set(1);
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

    pub(crate) fn tick(&mut self, state: &mut Box<dyn State>, io: &mut IORegs, lcd: &mut Lcd) {
        self.cram.tick(io);
        let lcdc = LCDC(io.io(IO::LCDC).read());
        if self.lcdc.enabled() && !lcdc.enabled() {
            self.dots = 0;
            io.io(IO::LY).direct_write(0);
            self.set_state(io, state, Box::new(HState::last()));
            lcd.disable();
        }
        self.lcdc = lcdc;
        self.dots += 1;
        if self.lcdc.enabled() {
            if self.regs.ly.read() == self.regs.lyc.read() { self.regs.stat.set(2); } else { self.regs.stat.reset(2); }
            if let Some(next) = state.tick(self, io, lcd) {
                let mode = next.mode();
                if mode == Mode::VBlank {
                    self.dots = 0;
                    lcd.enable();
                }
                self.set_state(io, state, next);
            }
        } else if self.dots == 65664 {
            self.set_state(io, state, VState::immediate().boxed());
        }
    }

    pub fn sprite(&self, index: usize) -> Sprite {
        self.oam().inner().sprites[index]
    }

    pub fn set(&mut self, lcd: &mut Lcd, io: &mut IORegs, lx: usize, ly: usize, pixel: Pixel) {
        lcd.set(lx, ly, self.cram.color(pixel, io));
    }

    pub fn default_state() -> PpuState { VState::new().boxed() }
}

impl Device for Ppu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.cram.configure(bus);
        self.vram_mut().configure(bus);
    }
}
