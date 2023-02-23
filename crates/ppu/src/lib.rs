#![feature(slice_flatten)]
#![feature(generic_const_exprs)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use std::collections::vec_deque::VecDeque;

use std::fmt::{Debug, Formatter, Write};
use std::rc::Rc;
use lcd::{LCD, Lcd};
use mem::{Oam, oam::Sprite, Vram};
use shared::io::{IO, IOReg, LCDC};
use shared::mem::{*, Device, IOBus, Mem, PPU};
use shared::utils::Cell;
use shared::utils::image::Image;
use crate::fetcher::Fetcher;

mod fetcher;
mod render;

struct REdge {
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
pub struct Registers {
    lcdc: IOReg,
    stat: IOReg,
    scy: IOReg,
    scx: IOReg,
    ly: IOReg,
    lyc: IOReg,
    bgp: IOReg,
    obp0: IOReg,
    obp1: IOReg,
    wy: IOReg,
    wx: IOReg,
    bcps: IOReg,
    bcpd: IOReg,
    ocps: IOReg,
    ocpd: IOReg,
    opri: IOReg,
    interrupt: IOReg
}

impl Device for Registers {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.lcdc = bus.io(IO::LCDC);
        self.stat = bus.io(IO::STAT);
        self.scx = bus.io(IO::SCX);
        self.scy = bus.io(IO::SCY);
        self.ly = bus.io(IO::LY);
        self.lyc = bus.io(IO::LYC);
        self.bgp = bus.io(IO::BGP);
        self.obp0 = bus.io(IO::OBP0);
        self.obp1 = bus.io(IO::OBP1);
        self.wx = bus.io(IO::WX);
        self.bcps = bus.io(IO::BCPS);
        self.bcpd = bus.io(IO::BCPD);
        self.ocps = bus.io(IO::OCPS);
        self.ocpd = bus.io(IO::OCPD);
        self.opri = bus.io(IO::OPRI);
        self.interrupt = bus.io(IO::IF);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    Search = 2,
    Transfer = 3,
    HBlank = 0,
    VBlank = 1,
}

trait State: Debug {
    fn mode(&self) -> Mode;
    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>>;
    fn boxed(self) -> Box<dyn State> where  Self: 'static + Sized { Box::new(self) }
    fn name(&self) -> String {
        format!("{:?}", self.mode())
    }
}

#[derive(Default)]
struct Window {
    scan_enabled: bool,
    enabled: bool,
    x: u8,
    y: u8
}

const TILEDATA_WIDTH: usize = 16 * 16;
const TILEDATA_HEIGHT: usize = 24 * 16;

#[derive(Default)]
struct Scroll {
    x: u8,
    y: u8
}

pub(crate) struct Ppu {
    tile_cache: HashSet<usize>,
    dots: usize,
    oam: Oam,
    vram: Vram,
    state: Box<dyn State>,
    regs: Registers,
    sprites: Vec<Sprite>,
    lcd: Lcd,
    lcdc: LCDC,
    win: Window,
    cgb: bool,
    sc: Scroll,
    stat: REdge,
    tiledata: Vec<Image<TILEDATA_WIDTH, TILEDATA_HEIGHT>>
}

#[derive(Debug)]
struct OamState {
    clock: u8,
    sprite: usize
}

impl OamState {
    fn new() -> Self { Self { sprite: 0, clock: 0 } }
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub color: u8,
    pub palette: u8,
    pub index: Option<u8>,
    pub priority: Option<bool>
}

impl Pixel {

    pub fn color(&self) -> [u8; 3] {
        const COLORS: [[u8; 3]; 4] = [[0xBF; 3], [0x7F; 3], [0x3F; 3], [0; 3]];
        let color = (self.palette >> (2 * self.color)) & 0x3;
        COLORS[color as usize]
    }

    pub fn white(palette: u8) -> Self {
        Self {
            color: 0,
            palette,
            index: None,
            priority: None
        }
    }

    /// sprite priority mix
    pub fn mix(&mut self, rhs: Pixel) {
        *self = match (self.color, rhs.color, self.index, rhs.index) {
            (_, _, None, Some(_)) => rhs,
            (_, _, Some(_), None ) => *self,
            (_, 0, ..) => *self,
            (0, ..) => rhs,
            (_, _, Some(x1), Some(x2) ) if x1 < x2 => *self,
            (_, _, Some(x1), Some(x2) ) if x1 > x2 => rhs,
            _ => *self,
        }
    }
}

pub struct ObjFifo {
    inner: VecDeque<Pixel>,
}

trait Fifo {
    fn push(&mut self, data: Vec<Pixel>) -> bool;
}

impl ObjFifo {
    pub fn new() -> Self {
        ObjFifo { inner: VecDeque::with_capacity(8) }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        self.inner.pop_front()
    }

    fn merge(&mut self, data: Vec<Pixel>) -> bool {
        for _ in self.inner.len()..8 {
            self.inner.push_back(Pixel {
                color: 0x00,
                palette: 0x00,
                index: None,
                priority: None
            });
        }
        self.inner
            .iter_mut()
            .zip(data.into_iter())
            .for_each(|(obj, p)| { obj.mix(p); });
        true
    }
}

pub struct BgFifo {
    inner: VecDeque<Pixel>,
    enabled: bool
}

impl BgFifo {
    pub fn new() -> Self {
        BgFifo {
            enabled: false,
            inner: VecDeque::with_capacity(16)
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.disable();
    }

    pub(crate) fn mix(&mut self, oam: &mut ObjFifo, ppu: &mut Ppu) -> Option<Pixel> {
        if self.enabled {
            let res = match (oam.pop(), self.inner.pop_front()) {
                (None, Some(bg)) => Some(bg),
                (Some(oam), Some(bg)) => {
                    Some({
                        let bg = if !ppu.cgb && !ppu.lcdc.priority() { Pixel::white(ppu.regs.bgp.read()) } else { bg };
                        if oam.color == 0x0 { bg }
                        else if ppu.lcdc.priority() && bg.color != 0 && (oam.priority.unwrap_or(false) || bg.priority.unwrap_or(false)) { bg }
                        else { oam }
                    })
                },
                (_, None) => unreachable!()
            };
            if self.inner.len() <= 8 {
                self.disable();
            }
            res
        } else { None }
    }

    fn push(&mut self, data: Vec<Pixel>) -> bool {
        if self.inner.len() > 8 { return false };
        for pix in data {
            self.inner.push_back(pix);
        }
        if self.inner.len() > 8 { self.enable(); }
        true
    }
}

struct TransferState {
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
struct HState {
    dots: usize
}

#[derive(Debug)]
struct VState {
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
            ppu.win.scan_enabled = ppu.regs.wy.read() == ly;
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
    fn new(ppu: &Ppu) -> Self where Self: Sized {
        let ly = ppu.regs.ly.read();
        let scx = ppu.regs.scx.read() & 0x7;
        Self { sprite: None, dots: 0, lx: 0, ly, scx, fetcher: Fetcher::new(ly / 8, ly % 8), bg: BgFifo::new(), oam: ObjFifo::new() }
    }
}

impl State for TransferState {
    fn mode(&self) -> Mode { Mode::Transfer }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        if ppu.win.scan_enabled && ppu.regs.wx.read() <= self.lx + 7 {
            if ppu.lcdc.win_enable() && !ppu.win.enabled {
                println!("window");
                self.scx = self.scx.saturating_sub(1);
                self.fetcher.set_mode(fetcher::Mode::Window, self.lx);
                self.bg.clear();
            }
            ppu.win.enabled = ppu.lcdc.win_enable();
        }
        self.fetcher.tick(ppu, &mut self.bg, &mut self.oam);
        if self.scx == 0 && ppu.lcdc.obj_enable() && self.bg.enabled && !self.fetcher.fetching_sprite() {
            let idx = if let Some(sprite) = self.sprite { sprite + 1 } else { 0 };
            for i in idx..ppu.sprites.len() {
                if ppu.sprites[i].screen_x() == self.lx || (ppu.sprites[i].x < 8 && self.lx == 0) {
                    self.sprite = Some(i);
                    self.fetcher.set_mode(fetcher::Mode::Sprite(ppu.sprites[i], self.lx), self.lx);
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
            ppu.lcd.set(self.lx as usize, self.ly as usize, pixel.color());
            self.sprite = None;
            self.lx += 1;
            if self.lx == 160 {
                return Some(HState::new(376 - self.dots).boxed())
            }
        }
        None
    }
}

impl VState {
    const DOTS: usize = 4560;

    pub fn new() -> Self { Self { dots: Self::DOTS } }
}

impl State for VState {
    fn mode(&self) -> Mode { Mode::VBlank }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        if self.dots == Self::DOTS {
            ppu.regs.interrupt.set(0);
        }
        self.dots -= 1;
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
}

impl State for HState {
    fn mode(&self) -> Mode { Mode::HBlank }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        self.dots -= 1;
        if self.dots == 0 {
            let ly = ppu.regs.ly.read() + 1;
            ppu.regs.ly.direct_write(ly);
            Some(if ly == 144 { VState::new().boxed() } else { OamState::new().boxed() })
        } else {
            None
        }
    }
}

impl Ppu {
    pub fn new(cgb: bool, lcd: Lcd) -> Self {
        let mut sprites = Vec::with_capacity(10);
        Self {
            sc: Scroll::default(),
            tile_cache: HashSet::with_capacity(384),
            dots: 0,
            oam: Oam::new(),
            vram: Vram::new(cgb),
            regs: Registers::default(),
            state: Box::new(VState::new()),
            sprites,
            lcd,
            cgb,
            lcdc: LCDC(0),
            win: Default::default(),
            tiledata: vec![],
            stat: REdge::new()
        }
    }

    fn tick(&mut self) {
        let lcdc = LCDC(self.regs.lcdc.read());
        if self.lcdc.enabled() && !lcdc.enabled() {
            self.dots = 0;
            self.regs.ly.direct_write(0);
            self.state = Box::new(OamState::new());
            self.lcd.disable();
        }
        self.lcdc = lcdc;
        if !self.lcdc.enabled() { return; }
        if self.regs.ly.read() == self.regs.lyc.read() { self.regs.stat.set(2); } else { self.regs.stat.reset(2); }
        let mut state = std::mem::replace(&mut self.state, Box::new(OamState::new()));
        self.state = if let Some(next) = state.tick(self) {
            let mode = next.mode();
            if mode == Mode::VBlank {
                self.dots = 0;
                self.lcd.enable();
            }
            next
        } else { state };
        let mode = self.state.mode();
        // TODO move this back to next state.
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
}

impl Device for Ppu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.regs.configure(bus);
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

pub struct Controller {
    tab: render::Tabs,
    init: bool,
    clock: usize,
    storage: HashMap<render::Textures, shared::egui::TextureHandle>,
    ppu: Rc<RefCell<Ppu>>
}

impl Controller {
    pub fn new(cgb: bool, lcd: Lcd) -> Self {
        Self {
            tab: render::Tabs::Oam,
            clock: 0,
            init: false,
            storage: HashMap::with_capacity(256),
            ppu: Ppu::new(cgb, lcd).cell()
        }
    }

    pub fn tick(&mut self) {
        self.ppu.as_ref().borrow_mut().tick();
    }
}

impl Device for Controller {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ppu.as_ref().borrow_mut().configure(bus);
    }
}

impl PPU for Controller {
    fn vram(&self) -> Rc<RefCell<dyn Mem>> {
        self.ppu.clone()
    }

    fn oam(&self) -> Rc<RefCell<dyn Mem>> {
        self.ppu.clone()
    }
}
