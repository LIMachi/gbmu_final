use mem::oam::Sprite;
use shared::io::LCDC;
use shared::mem::{Mem, VRAM};
use crate::{Pixel, Ppu};

#[derive(Debug)]
enum State {
    Tile,
    DataLow,
    DataHigh,
    Sleep,
    Push
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Mode {
    Bg,
    Window,
    Sprite(Sprite, u8)
}

pub struct Fetcher {
    clock: u8,
    x: u8,
    y: u8,
    scanline: u8,
    flip_x: bool,
    flip_y: bool,
    tile: u16,
    mode: Mode,
    prev: Mode,
    low: Option<u8>,
    high: Option<u8>,
    state: State,
}

const TILE_MAP_1: u16 = 0x9800;
const TILE_MAP_2: u16 = 0x9C00;
const RELATIVE: u16 = 0x9000;

impl Fetcher {
    const WIN_OFFSET: u8 = 7;

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
            prev: Mode::Bg,
            low: None,
            high: None
        }
    }

    pub fn set_mode(&mut self, mode: Mode, lx: u8) -> bool {
        let mut changed = false;
        if mode == Mode::Window && self.mode == Mode::Bg {
            self.state = State::Tile;
            self.x = lx;
            changed = true;
        }
        if let Mode::Sprite(Sprite { tile, flags, .. }, .. ) = mode {
            self.tile = tile as u16;
            self.state = State::DataLow;
            self.flip_x = (flags & 0x20) != 0;
            self.flip_y = (flags & 0x40) != 0;
        }
        self.mode = mode;
        changed
    }

    fn window_active(&self) -> bool {
        self.mode == Mode::Window
    }

    // TODO fix window y. rest looks cool.
    fn get_tile(&mut self, ppu: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::Tile;
        }
        self.clock = 0;
        self.flip_y = false;
        self.flip_x = false;
        let lcdc = LCDC(ppu.regs.lcdc.read());
        let ly = ppu.regs.ly.read();
        let scx = ppu.regs.scx.read();
        let scy = ppu.regs.scy.read();
        let addr = 0x1800 | match (self.window_active(), lcdc.bg_area(), lcdc.win_area()) {
            (false, n , _) => (n as u16) << 10 | ((ly.wrapping_add(scy) as u16 / 8) << 5) | ((self.x + scx / 8) & 0x1F) as u16, //bg tile (LCDC.3)
            (true, _, n) => (n as u16) << 10 | (ppu.win.y << 5) as u16 | self.x as u16, //window tile (LCDC.6)
        };
        let bank = if ppu.cgb {
            if let Mode::Sprite(Sprite { flags, .. }, _) = self.mode { (flags >> 2) & 0x1 } else { 0 }
        } else { 0 };
        self.tile = ppu.vram.read_bank(addr, bank as usize) as u16;
        State::DataLow
    }

    fn get_tile_data(&self, ppu: &Ppu, high: bool) -> u8 {
        let lcdc = ppu.lcdc;
        let scy = ppu.regs.scy.read();
        let ly = ppu.regs.ly.read();
        let (wrap, tile) = if lcdc.obj_tall() { (0xF, self.tile & 0xFE)  } else { (0x7, self.tile) };
        let y = (match self.mode {
            Mode::Bg => ly.wrapping_add(scy),
            Mode::Window => ppu.win.y,
            Mode::Sprite(s, ..) => ly.wrapping_sub(s.y),
        } & wrap) as u16;
        let y = if self.flip_y { wrap as u16 - y } else { y };
        let addr = (match self.mode {
            Mode::Bg | Mode::Window => !(!lcdc.relative_addr() || (tile & 0x80) != 0) as u16,
            Mode::Sprite( .. ) => 0
        } << 12) | (tile << 4) | (y << 1) | (high as u16);
        let bank = match self.mode {
            Mode::Sprite(sprite, _) if ppu.cgb => (sprite.flags >> 2) & 0x1,
            _ => 0
        };
        ppu.vram.read_bank(addr, bank as usize)
    }

    fn data_low(&mut self, ppu: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::DataLow;
        }
        self.clock = 0;
        self.low = Some(self.get_tile_data(ppu, false));
        State::DataHigh
    }

    fn data_high(&mut self, ppu: &Ppu, fifo: &mut super::BgFifo, oam: &mut super::ObjFifo) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::DataHigh;
        }
        self.clock = 0;
        self.high = Some(self.get_tile_data(ppu, true));
        match self.push(ppu, fifo, oam) {
            State::Push => State::Sleep,
            State::Tile => State::Tile,
            _ =>unreachable!()
        }
    }

    fn push(&mut self, ppu: &Ppu, bg: &mut super::BgFifo, oam: &mut super::ObjFifo) -> State {
        if let (Some(low), Some(high)) = (self.low.take(), self.high.take()) {
            let dmg = ppu.regs.bgp.read();
            let index = if let Mode::Sprite(_, x) = self.mode { Some(x) } else { None };
            let priority = if let Mode::Sprite(sp, _) = self.mode { Some((sp.flags >> 7) != 0) } else { None };
            // TODO shift some pixels if it's too far left (<8)
            let mut colors = [0; 8];
            colors.iter_mut().enumerate().for_each(|(i, c)| {
                let x = 7 - i;
                *c = ((low >> x) & 0x1) | (((high >> x) & 0x1) << 1);
            });
            if self.flip_x { colors.reverse(); }
            if let Mode::Sprite(sp, n) = self.mode {
                let s = 8u8.saturating_sub(sp.x) as usize;
                colors.rotate_left(s);
                colors[8-s..].iter_mut().for_each(|x| *x = 0);
                oam.merge(colors.into_iter().map(|x| Pixel { color: x, palette: dmg, index, priority }).collect());
                self.set_mode(self.prev, self.x);
                bg.enable();
            } else if bg.push(colors.into_iter().map(|x| Pixel { color: x, palette: dmg, index, priority }).collect()) {
                self.x += 1;
                self.low = Some(low);
                self.high = Some(high);
            } else { return State::Push }
        }
        State::Tile
        // TODO use obj palette if mode == sprite
    }

    fn sleep(&mut self, _: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::Sleep;
        }
        self.clock = 0;
        State::Push
    }

    pub fn fetching_sprite(&self) -> bool {
        matches!(self.mode, Mode::Sprite( .. ))
    }

    pub(crate) fn tick(&mut self, ppu: &Ppu, bg: &mut super::BgFifo, oam: &mut super::ObjFifo) {
        self.state = match self.state {
            State::Tile => self.get_tile(ppu),
            State::DataLow => self.data_low(ppu),
            State::DataHigh => self.data_high(ppu, bg, oam),
            State::Sleep => self.sleep(ppu),
            State::Push => self.push(ppu, bg, oam),
        };
    }
}
