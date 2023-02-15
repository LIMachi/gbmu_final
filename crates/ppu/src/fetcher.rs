use shared::io::LCDC;
use shared::mem::{Mem, VRAM};
use crate::{Pixel, Ppu};

enum State {
    Tile,
    DataLow,
    DataHigh,
    Sleep,
    Push
}

enum Mode {
    Bg,
    Sprite(usize)
}

pub struct Fetcher {
    clock: u8,
    x: u8,
    y: u8,
    scanline: u8,
    flip_x: bool,
    flip_y: bool,
    tile: u8,
    mode: Mode,
    tile_data: Option<u16>,
    state: State,
}

const TILE_MAP_1: u16 = 0x9800;
const TILE_MAP_2: u16 = 0x9C00;
const RELATIVE: u16 = 0x9000;

impl Fetcher {

    pub fn new(tile_y: u8, scanline: u8) -> Self {
        Fetcher {
            clock: 0,
            state: State::Tile,
            x: 0,
            y: tile_y,
            scanline,
            flip_x: false,
            flip_y: false,
            tile: 0,
            mode: Mode::Bg,
            tile_data: None
        }
    }

    fn get_tile(&mut self, ppu: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::Tile;
        }
        self.clock = 0;
        let lcdc = LCDC(ppu.regs.lcdc.read());
        let ly = ppu.regs.ly.read();
        let scx = ppu.regs.scx.read();
        let scy = ppu.regs.scx.read();
        let addr = match (ppu.flags.window_active(), lcdc.bg_area(), lcdc.win_area()) {
            (false, true, _) => TILE_MAP_2, //bg tile, relative address (LCDC.3)
            (false, false, _) => TILE_MAP_1, //bg tile, absolute address (LCDC.3)
            (true, _, true) => TILE_MAP_2, //window tile, relative address (LCDC.6)
            (true, _, false) => TILE_MAP_1, //window tile, absolute address (LCDC.6)
        };
        let (x, y) = if ppu.flags.window_active() {
            (((scx / 8) + self.x) & 0x1F, ly.wrapping_add(scy))
        } else { (self.x, self.y) };
        let addr = addr + x as u16 + y as u16 * 0x20;
        self.tile = ppu.vram.read_bank(addr - VRAM, 0);
        self.x += 1;
        State::DataLow
    }

    fn get_tile_data(&self, ppu: &Ppu, high: bool) -> u8 {
        let y = if self.flip_y { 7 - self.scanline } else { self.scanline } as u16;
        let mut addr = 2 * y + if LCDC(ppu.regs.lcdc.read()).relative_addr() {
            let i = self.tile as i8;
            if i < 0 { RELATIVE - (-i as u16) * 16 } else { RELATIVE + i as u16 * 16 }
        } else { self.tile as u16 * 16 + VRAM };
        if high { addr += 1 };
        ppu.vram.read_bank(addr - VRAM, 0)
    }

    fn data_low(&mut self, ppu: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::DataLow;
        }
        self.clock = 0;
        self.tile_data = Some(self.get_tile_data(ppu, false) as u16);
        State::DataHigh
    }

    fn data_high(&mut self, ppu: &Ppu, fifo: &mut super::Fifo) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::DataHigh;
        }
        self.clock = 0;
        self.tile_data = Some(self.tile_data.unwrap() | (self.get_tile_data(ppu, true) as u16) << 8);
        self.push(ppu, fifo);
        State::Sleep
    }

    fn push(&mut self, ppu: &Ppu, fifo: &mut super::Fifo) -> State {
        if self.tile_data.is_none() { return State::Tile };
        let tile_data = self.tile_data.unwrap();
        // TODO use obj palette if mode == sprite
        let mut iter = (0..8).into_iter().map(|x| (tile_data >> (2 * x)) & 0x3)
            .map(|x| Pixel {
                color: x as u8,
                palette: ppu.regs.bgp.read(),
                index: None,
                priority: None
            });
        let pixels: Vec<Pixel> = if self.flip_x { iter.rev().collect() } else { iter.collect() };
        if fifo.push(pixels) {
            self.tile_data = None;
            State::Tile
        } else {
            State::Push
        }
    }

    fn sleep(&mut self, _: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::Sleep;
        }
        self.clock = 0;
        State::Push
    }

    pub fn tick(&mut self, ppu: &Ppu, fifo: &mut super::Fifo) {
        self.state = match self.state {
            State::Tile => self.get_tile(ppu),
            State::DataLow => self.data_low(ppu),
            State::DataHigh => self.data_high(ppu, fifo),
            State::Sleep => self.sleep(ppu),
            State::Push => self.push(ppu, fifo),
        };
    }
}
