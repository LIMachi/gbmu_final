use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use log::info;
use lcd::Framebuffer;
use mem::{Oam, Vram, oam::Sprite};
use shared::io::{IO, IOReg, LCDC};
use shared::mem::{Device, IOBus, Mem, PPU, *};
use shared::utils::Cell;
use crate::fetcher::Fetcher;

mod fetcher;
//
// pub struct tiles {
//     IdColor:
//     squares:
//     Background:
//     Window:
//     Obj:
// }

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

#[repr(u8)]
pub enum Mode {
    Search = 2,
    Transfer = 3,
    HBlank = 0,
    VBlank = 1,
}

trait State {
    fn mode(&self) -> Mode;
    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>>;
    fn boxed(self) -> Box<dyn State> where  Self: 'static + Sized { Box::new(self) }
}

#[derive(Default)]
struct Flags {
    window: u8
}

impl Flags {
    fn window_active(&self) -> bool { self.window == 0x7 }
}

struct Ppu {
    oam: Oam,
    vram: Vram,
    state: Box<dyn State>,
    regs: Registers,
    sprites: Vec<Sprite>,
    buffer: Rc<RefCell<dyn Framebuffer>>,
    flags: Flags,
    lcdc: LCDC
}

struct OamState {
    clock: u8,
    sprite: usize
}

impl OamState {
    fn new() -> Self { Self { sprite: 0, clock: 0 } }
}

pub struct Pixel {
    pub color: u8,
    pub palette: u8,
    pub index: Option<usize>,
    pub priority: Option<bool>
}

impl Pixel {
    pub fn color(&self) -> [u8; 3] {
        let color = (self.palette >> (2 * self.color)) & 0x3;
        [color * 64; 3]
    }
}

pub struct Fifo {
    inner: VecDeque<Pixel>,
}

impl Fifo {
    pub fn new() -> Self {
        Fifo { inner: VecDeque::with_capacity(16) }
    }

    pub fn push(&mut self, data: Vec<Pixel>) -> bool {
        if self.inner.len() > 8 { return false };
        for pix in data {
            self.inner.push_back(pix);
        }
        true
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        self.inner.pop_front()
    }
}

struct TransferState {
    dots: usize,
    x: usize,
    y: usize,
    fetcher: Fetcher,
    bg: Fifo,
    oam: Fifo,
}

struct HState {
    dots: usize
}

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
        let oam = ppu.oam.sprites[self.sprite];
        if ly + 16 >= oam.y && ly + 8 < oam.y && ppu.sprites.len() < 10 {
            ppu.sprites.push(oam);
        }
        self.sprite += 1;
        if self.sprite == 40 { Some(TransferState::new(ppu).boxed()) } else { None }
    }
}

impl TransferState {
    fn new(ppu: &Ppu) -> Self where Self: Sized {
        let ly = ppu.regs.ly.read();
        Self { dots: 0, x: 0, y: ly as usize, fetcher: Fetcher::new(ly / 8, ly % 8), bg: Fifo::new(), oam: Fifo::new() }
    }

    fn fetch(&mut self, ppu: &mut Ppu) -> Option<Pixel> {
        self.bg.pop()
    }
}

impl State for TransferState {
    fn mode(&self) -> Mode { Mode::Transfer }

    fn tick(&mut self, ppu: &mut Ppu) -> Option<Box<dyn State>> {
        self.dots += 1;
        self.fetcher.tick(ppu, &mut self.bg);
        // TODO check every sprite for this x.
        if let Some(pixel) = self.fetch(ppu) {
            ppu.buffer.as_ref().borrow_mut().set(self.x, self.y, pixel.color());
            self.x += 1;
            // TODO render pixel to LCD
            if self.x == 160 {
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
    pub fn new(cgb: bool, buffer: Rc<RefCell<dyn Framebuffer>>) -> Self {
        let mut sprites = Vec::with_capacity(10);
        Self {
            oam: Oam::new(),
            vram: Vram::new(cgb),
            regs: Registers::default(),
            state: Box::new(OamState::new()),
            sprites,
            buffer,
            flags: Default::default(),
            lcdc: LCDC(0),
        }
    }

    fn tick(&mut self) {
        let lcdc = LCDC(self.regs.lcdc.read());
        if self.lcdc.enabled() && !lcdc.enabled() {
            self.regs.ly.direct_write(0);
            self.state = Box::new(HState::new(1));
        }
        self.lcdc = lcdc;
        if !self.lcdc.enabled() { return; };
        let mut state = std::mem::replace(&mut self.state, Box::new(OamState::new()));
        self.state = if let Some(state) = state.tick(self) {
            state
        } else { state };
    }
}

impl Device for Ppu {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.regs.configure(bus);
    }
}

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
            VRAM..=VRAM_END => { self.vram.write(addr, value, absolute) },
            _ => unreachable!()
        }
    }
}

pub struct Controller {
    ppu: Rc<RefCell<Ppu>>
}

impl Controller {
    pub fn new(cgb: bool, lcd: &lcd::Lcd) -> Self {
        Self {
            ppu: Ppu::new(cgb, lcd.framebuffer()).cell()
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
