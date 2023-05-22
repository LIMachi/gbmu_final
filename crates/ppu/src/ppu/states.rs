use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use lcd::Lcd;
use mem::oam::Sprite;
use shared::io::{IO, IORegs, LCDC};
use shared::mem::Source;

use super::{
    fetcher::{self, Fetcher}, fifo::{BgFifo, ObjFifo}, Ppu, Scroll,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Mode {
    Search = 2,
    Transfer = 3,
    HBlank = 0,
    VBlank = 1,
}

pub(crate) trait State: Debug {
    fn mode(&self) -> Mode;
    fn tick(&mut self, ppu: &mut Ppu, io: &mut IORegs, lcd: &mut Lcd) -> Option<Box<dyn State>>;
    fn boxed(self) -> Box<dyn State> where Self: 'static + Sized { Box::new(self) }
    fn name(&self) -> String { format!("{:?}", self.mode()) }
    fn first_tick(&self) -> bool { false }
    fn raw(&self) -> Vec<u8> { vec![] }
}

#[derive(Debug)]
pub struct OamState {
    clock: u8,
    sprite: usize,
}

impl OamState {
    pub fn new() -> Self { Self { sprite: 0, clock: 0 } }

    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn State> {
        Box::new(Self { clock: raw[0], sprite: raw[1] as usize })
    }
}

impl State for OamState {
    fn mode(&self) -> Mode { Mode::Search }

    fn tick(&mut self, ppu: &mut Ppu, io: &mut IORegs, _: &mut Lcd) -> Option<Box<dyn State>> {
        self.clock += 1; // we only tick one every 2 clock cycle
        if self.clock < 2 { return None; }
        self.clock = 0;
        let ly = io.io(IO::LY).value();
        if self.sprite == 0 {
            ppu.sc = Scroll::default();
            ppu.sprites.clear();
            ppu.win.scan_enabled = io.io(IO::WY).value() <= ly;
            ppu.win.x = 0;
        }
        let y = ppu.oam().get(Source::Ppu, |oam| oam.sprites[self.sprite].y);
        if ly + if ppu.lcdc.obj_tall() { 0 } else { 8 } < y && ly + 16 >= y && ppu.sprites.len() < 10 {
            ppu.sprites.push(self.sprite);
        }
        self.sprite += 1;
        if self.sprite == 40 {
            ppu.sc.x = ppu.sc.x.max(io.io(IO::SCX).value());
            ppu.sc.y = ppu.sc.y.max(io.io(IO::SCY).value());
            Some(TransferState::new(ppu, io).boxed())
        } else { None }
    }

    fn raw(&self) -> Vec<u8> {
        vec![self.clock, self.sprite as u8]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferState {
    dots: usize,
    lx: u8,
    ly: u8,
    scx: u8,
    fetcher: Fetcher,
    bg: BgFifo,
    oam: ObjFifo,
    sprite: Option<usize>,
}

impl Debug for TransferState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[{} - {}]({}) left {}", self.lx, self.ly, self.scx, self.dots).as_str())
    }
}

impl TransferState {
    pub(crate) fn new(_ppu: &Ppu, io: &IORegs) -> Self where Self: Sized {
        let ly = io.io(IO::LY).value();
        let scx = io.io(IO::SCX).value() & 0x7;
        Self {
            sprite: None,
            dots: 0,
            lx: 0,
            ly,
            scx,
            fetcher: Fetcher::new(ly),
            bg: BgFifo::new(),
            oam: ObjFifo::new(io.io(IO::OPRI).bit(0) != 0),
        }
    }

    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn State> {
        let t: Self = bincode::deserialize(&*raw).unwrap();
        Box::new(t)
    }
}

impl State for TransferState {
    fn mode(&self) -> Mode { Mode::Transfer }

    fn tick(&mut self, ppu: &mut Ppu, io: &mut IORegs, lcd: &mut Lcd) -> Option<Box<dyn State>> {
        self.dots += 1;
        let wx = io.io(IO::WX).value();
        if ppu.win.scan_enabled && wx <= self.lx + 7 {
            if ppu.lcdc.win_enable() && !ppu.win.enabled {
                self.scx = 7u8.saturating_sub(wx);
                self.fetcher.set_mode(fetcher::Mode::Window);
                self.bg.clear();
                ppu.win.enabled = true;
            }
        }
        self.fetcher.tick(ppu, io, &mut self.bg, &mut self.oam);
        if self.scx == 0 && ppu.lcdc.obj_enable() && self.bg.enabled() && !self.fetcher.fetching_sprite() {
            let idx = if let Some(sprite) = self.sprite { sprite + 1 } else { 0 };
            for i in idx..ppu.sprites.len() {
                let idx = ppu.sprites[i];
                let sprite = ppu.oam_mut().get_mut(Source::Ppu)
                    .map(|x| x.sprites[idx])
                    .unwrap_or_else(|| Sprite::unavailable());
                if sprite.screen_x() == self.lx || (sprite.x != 0 && sprite.x < 8 && self.lx == 0) {
                    self.sprite = Some(i);
                    self.fetcher.set_mode(fetcher::Mode::Sprite(sprite, i as u8));
                    self.bg.disable();
                    break;
                }
            }
        }
        if let Some(pixel) = self.bg.mix(&mut self.oam, io) {
            if self.scx > 0 {
                self.scx -= 1;
                return None;
            }
            ppu.set(lcd, io, self.lx as usize, self.ly as usize, pixel);
            self.sprite = None;
            self.lx += 1;
            if self.lx == 160 {
                if ppu.win.enabled { ppu.win.y += 1; }
                ppu.win.enabled = false;
                if self.dots > 289 {
                    log::warn!("transfer took {} dots", self.dots);
                }
                return Some(HState::new(376usize.saturating_sub(self.dots)).boxed());
            }
        }
        None
    }

    fn raw(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[derive(Debug)]
pub struct VState {
    dots: usize,
}

impl VState {
    const DOTS: usize = 4560;

    pub fn new() -> Self { Self { dots: Self::DOTS } }

    pub fn immediate() -> Self {
        Self { dots: 0 }
    }

    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn State> {
        Box::new(Self { dots: raw[0] as usize | ((raw[1] as usize) << 8) })
    }
}

impl State for VState {
    fn mode(&self) -> Mode { Mode::VBlank }

    fn tick(&mut self, ppu: &mut Ppu, io: &mut IORegs, _: &mut Lcd) -> Option<Box<dyn State>> {
        if self.dots == Self::DOTS {
            io.int_set(0);
        } else if self.dots == 0 {
            return Some(OamState::new().boxed());
        }
        let ly = io.io_mut(IO::LY);
        self.dots = self.dots.saturating_sub(1);
        if self.dots == 0 {
            ppu.win.y = 0;
            ly.direct_write(0);
            Some(OamState::new().boxed())
        } else {
            if self.dots % 456 == 0 {
                let v = (ly.value() + 1) % 153;
                ly.direct_write(v);
            }
            None
        }
    }

    fn first_tick(&self) -> bool {
        self.dots >= Self::DOTS
    }

    fn raw(&self) -> Vec<u8> {
        vec![self.dots as u8, (self.dots >> 8) as u8]
    }
}

#[derive(Debug)]
pub struct HState {
    dots: usize,
}

impl HState {
    pub fn new(dots: usize) -> Self {
        Self { dots }
    }

    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn State> {
        Box::new(Self { dots: raw[0] as usize | ((raw[1] as usize) << 8) })
    }
}

impl State for HState {
    fn mode(&self) -> Mode { Mode::HBlank }

    fn tick(&mut self, _ppu: &mut Ppu, io: &mut IORegs, _: &mut Lcd) -> Option<Box<dyn State>> {
        self.dots = self.dots.saturating_sub(1);
        if self.dots == 0 {
            let reg = io.io_mut(IO::LY);
            let ly = reg.value() + 1;
            reg.direct_write(ly);
            Some(if ly == 144 {
                VState::new().boxed()
            } else {
                OamState::new().boxed()
            })
        } else {
            None
        }
    }

    fn raw(&self) -> Vec<u8> {
        vec![self.dots as u8, (self.dots >> 8) as u8]
    }
}
