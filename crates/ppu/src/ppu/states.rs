use std::fmt::{Debug, Formatter, Write};
use super::{
    Ppu, fifo::{BgFifo, ObjFifo}, Scroll, fetcher::{self, Fetcher}
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    Search = 2,
    Transfer = 3,
    HBlank = 0,
    VBlank = 1,
}

pub(crate) trait State: Debug {
    fn mode(&self) -> Mode;
    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>>;
    fn boxed(self) -> Box<dyn State> where  Self: 'static + Sized { Box::new(self) }
    fn name(&self) -> String {
        format!("{:?}", self.mode())
    }
}

#[derive(Debug)]
pub struct OamState {
    clock: u8,
    sprite: usize
}

impl OamState {
    pub fn new() -> Self { Self { sprite: 0, clock: 0 } }
}

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

#[derive(Debug)]
pub struct HState {
    dots: usize
}

#[derive(Debug)]
pub struct VState {
    dots: usize
}

impl State for OamState {
    fn mode(&self) -> Mode { Mode::Search }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        self.clock += 1; // we only tick one every 2 clock cycle
        if self.clock < 2 { return None }
        self.clock = 0;
        let ly = ppu.regs.ly.read();
        if self.sprite == 0 {
            ppu.sc = Scroll::default();
            ppu.sprites.clear();
            ppu.win.scan_enabled = ppu.regs.wy.read() <= ly;
        }
        let oam = ppu.oam.sprites[self.sprite];
        if ly + if ppu.lcdc.obj_tall() { 0 } else { 8 } < oam.y && ly + 16 >= oam.y && ppu.sprites.len() < 10 {
            ppu.sprites.push(oam);
        }
        self.sprite += 1;
        if self.sprite == 40 {
            ppu.sc.x = ppu.sc.x.max(ppu.regs.scx.read());
            ppu.sc.y = ppu.sc.y.max(ppu.regs.scy.read());
            Some(TransferState::new(ppu).boxed()) } else { None }
    }
}

impl TransferState {
    pub(crate) fn new(ppu: &Ppu) -> Self where Self: Sized {
        let ly = ppu.regs.ly.read();
        let scx = ppu.regs.scx.read() & 0x7;
        Self { sprite: None, dots: 0, lx: 0, ly, scx, fetcher: Fetcher::new(ly / 8, ly % 8), bg: BgFifo::new(), oam: ObjFifo::new() }
    }
}

impl State for TransferState {
    fn mode(&self) -> Mode { Mode::Transfer }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        self.dots += 1;
        if ppu.win.scan_enabled && ppu.regs.wx.read() <= self.lx + 7 {
            if ppu.lcdc.win_enable() && !ppu.win.enabled {
                // println!("win enabled for {} ({})", ppu.regs.ly.read(), ppu.win.y);
                self.scx = 7u8.saturating_sub(ppu.regs.wx.read());
                self.fetcher.set_mode(fetcher::Mode::Window);
                self.bg.clear();
                ppu.win.enabled = true;
            }
        }
        self.fetcher.tick(ppu, &mut self.bg, &mut self.oam);
        if self.scx == 0 && ppu.lcdc.obj_enable() && self.bg.enabled() && !self.fetcher.fetching_sprite() {
            let idx = if let Some(sprite) = self.sprite { sprite + 1 } else { 0 };
            for i in idx..ppu.sprites.len() {
                if ppu.sprites[i].screen_x() == self.lx || (ppu.sprites[i].x != 0 && ppu.sprites[i].x < 8 && self.lx == 0) {
                    self.sprite = Some(i);
                    self.fetcher.set_mode(fetcher::Mode::Sprite(ppu.sprites[i], self.lx));
                    self.bg.disable();
                    break ;
                }
            }
        }
        if let Some(pixel) = self.bg.mix(&mut self.oam, ppu) {
            if self.scx > 0 {
                self.scx -= 1;
                return None;
            }
            ppu.set(self.lx as usize, self.ly as usize, pixel);
            self.sprite = None;
            self.lx += 1;
            if self.lx == 160 {
                if ppu.win.enabled { ppu.win.y += 1; }
                ppu.win.enabled = false;
                return Some(HState::new(376usize.saturating_sub(self.dots)).boxed())
            }
        }
        None
    }
}

impl VState {
    const DOTS: usize = 4560;

    pub fn new() -> Self { Self { dots: Self::DOTS } }

    pub fn immediate() -> Self {
        Self { dots: 0 }
    }
}

impl State for VState {
    fn mode(&self) -> Mode { Mode::VBlank }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        if self.dots == Self::DOTS || self.dots == 0 {
            ppu.win.y = 0;
            ppu.regs.interrupt.set(0);
        }
        if self.dots == 0 {
            ppu.regs.ly.direct_write(0);
        }
        self.dots = self.dots.saturating_sub(1);
        if self.dots % 456 == 0 {
            let ly = (ppu.regs.ly.read() + 1) % 154;
            ppu.regs.ly.direct_write(ly);
        }
        if self.dots == 0 {
            Some(OamState::new().boxed())
        } else { None }
    }
}

impl HState {
    pub fn new(dots: usize) -> Self {
        Self { dots }
    }

    pub fn last() -> Self {
        Self { dots: 0 }
    }
}

impl State for HState {
    fn mode(&self) -> Mode { Mode::HBlank }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        self.dots = self.dots.saturating_sub(1);
        if self.dots == 0 {
            let ly = ppu.regs.ly.read() + 1;
            ppu.regs.ly.direct_write(ly);
            Some(if ly == 144 { VState::new().boxed() } else { OamState::new().boxed() })
        } else {
            None
        }
    }
}
