use serde::{Deserialize, Serialize};

use lcd::{Lcd, LCD};
use mem::{oam::{Oam, Sprite}, Vram};
use pixel::Pixel;
use shared::{io::{IO, IODevice, IORegs, LCDC}, egui::{epaint::ahash::{HashMap, HashMapExt}}};
use shared::mem::*;
use states::*;
use super::render::ColorBuffer;

mod fetcher;
mod cram;
mod pixel;
mod fifo;
pub(crate) mod states;

pub(crate) type PpuState = Box<dyn State>;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct REdge {
    inner: bool,
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

#[derive(Default, Serialize, Deserialize, Copy, Clone)]
pub(crate) struct Window {
    pub scan_enabled: bool,
    pub enabled: bool,
    pub y: u8,
    pub x: u8,
}

#[derive(Default, Serialize, Deserialize, Copy, Clone)]
pub(crate) struct Scroll {
    pub x: u8,
    pub y: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Ppu {
    pub(crate) cram: cram::CRAM,
    pub(crate) sprites: Vec<usize>,
    pub(crate) win: Window,
    pub(crate) sc: Scroll,
    pub(crate) stat: REdge,
    pub(crate) lcdc: u8,
    #[serde(default, skip)]
    pub(crate) oam: Option<&'static mut Lock<Oam>>,
    #[serde(default, skip)]
    pub(crate) vram: Option<&'static mut Lock<Vram>>,
    pub(crate) tile_cache: HashMap<usize, ColorBuffer<8, 8>>
}

impl Ppu {
    pub fn new() -> Self {
        let sprites = Vec::with_capacity(10);
        Self {
            sc: Scroll::default(),
            cram: cram::CRAM::default(),
            sprites,
            lcdc: 0,
            win: Default::default(),
            stat: REdge::new(),
            oam: None,
            vram: None,
            tile_cache: HashMap::with_capacity(768)
        }
    }

    pub(crate) fn mark(&mut self, tile: usize) {
        self.tile_cache.entry(tile).or_insert_with(ColorBuffer::<8, 8>::new);
    }

    pub(crate) fn store_tile(&mut self, io: &IORegs, tile: usize, y: u8, pixels: impl Iterator<Item=Pixel>) {
        self.tile_cache.entry(tile)
        .and_modify(|tile| {
            tile.set_line(y as usize, pixels);
        });
    }

    pub(crate) fn oam(&self) -> &Lock<Oam> { self.oam.as_ref().unwrap() }
    pub(crate) fn vram(&self) -> &Lock<Vram> { self.vram.as_ref().unwrap() }
    pub(crate) fn oam_mut(&mut self) -> &mut Lock<Oam> { self.oam.as_mut().unwrap() }
    pub(crate) fn vram_mut(&mut self) -> &mut Lock<Vram> { self.vram.as_mut().unwrap() }

    fn set_state(&mut self, regs: &mut IORegs, state: &mut Box<dyn State>, next: Box<dyn State>) {
        let mode = next.mode();
        let stat = regs.io_mut(IO::STAT);
        *state = next;
        stat.reset(0);
        stat.reset(1);
        if (mode as u8 & 0x1) != 0 { stat.set(0); }
        if (mode as u8 & 0x2) != 0 { stat.set(1); }
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

    /// safety guarantee: we will never hold oam/ vram longer than 'a
    pub(crate) fn claim<'a>(&mut self, oam: &'a mut Lock<Oam>, vram: &'a mut Lock<Vram>) {
        let oam = unsafe { std::mem::transmute::<&'a mut Lock<Oam>, &'static mut Lock<Oam>>(oam) };
        let vram = unsafe { std::mem::transmute::<&'a mut Lock<Vram>, &'static mut Lock<Vram>>(vram) };
        self.oam = Some(oam);
        self.vram = Some(vram);
    }

    /// safety guarantee: after self.claim<'a>, you must call this before exiting 'a scope.
    pub(crate) fn release(&mut self) {
        self.oam.take();
        self.vram.take();
    }

    pub(crate) fn tick(&mut self, state: &mut Box<dyn State>, io: &mut IORegs, lcd: &mut Lcd) {
        let lcdc = io.io(IO::LCDC).value();
        if self.lcdc.enabled() && !lcdc.enabled() {
            io.io_mut(IO::LY).direct_write(0);
            self.set_state(io, state, VState::immediate().boxed());
            lcd.disable(io);
        }
        self.lcdc = lcdc;
        if self.lcdc.enabled() {
            if let Some(next) = state.tick(self, io, lcd) {
                let mode = next.mode();
                if mode == Mode::VBlank {
                    lcd.vblank();
                    lcd.enable();
                }
                self.set_state(io, state, next);
            }
            if io.io(IO::LY).value() == io.io(IO::LYC).value() {
                io.io_mut(IO::STAT).set(2);
            } else {
                io.io_mut(IO::STAT).reset(2);
            }
            let mode = state.mode();
            let stat = io.io(IO::STAT);
            let input = stat.bit(3) != 0 && mode == Mode::HBlank;
            let input = input || (stat.bit(4) != 0 && mode == Mode::VBlank);
            let input = input || (stat.bit(5) != 0 && mode == Mode::Search);
            let input = input || (stat.bit(6) != 0 && stat.bit(2) != 0);
            if self.stat.tick(input) { io.int_set(1); }
        }
    }

    pub fn sprite(&self, index: usize) -> Sprite {
        self.oam().inner().sprites[index]
    }

    pub fn set(&mut self, lcd: &mut Lcd, io: &mut IORegs, lx: usize, ly: usize, pixel: Pixel) {
        lcd.set(lx, ly, self.cram.color(pixel, io));
    }

    pub(crate) fn default_state() -> PpuState { VState::new().boxed() }
}

impl IODevice for Ppu {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        self.cram.write(io, v, bus);
    }
}
