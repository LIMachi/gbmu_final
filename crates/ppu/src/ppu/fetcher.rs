use mem::oam::Sprite;
use shared::io::{CGB_MODE, LCDC};
use shared::mem::Source;
use crate::ppu::pixel::Attributes;
use super::{Pixel, Ppu, fifo::*};

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
    attrs: Attributes,
    tile: u16,
    mode: Mode,
    prev: Mode,
    low: Option<u8>,
    high: Option<u8>,
    state: State,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            clock: 0,
            state: State::Tile,
            x: 0,
            attrs: Attributes::default(),
            tile: 0,
            mode: Mode::Bg,
            prev: Mode::Bg,
            low: None,
            high: None
        }
    }

    pub fn set_mode(&mut self, mode: Mode) -> bool {
        let mut changed = false;
        if mode == Mode::Window && self.mode != Mode::Window {
            self.state = State::Tile;
            changed = true;
        }
        if let Mode::Sprite(Sprite { tile, flags, .. }, .. ) = mode {
            self.tile = tile as u16;
            self.state = State::DataLow;
            self.attrs = Attributes(flags);
            if let Mode::Sprite(..) = self.mode { } else { self.prev = self.mode };
        }
        self.mode = mode;
        changed
    }

    fn window_active(&self) -> bool {
        self.mode == Mode::Window
    }

    fn get_tile(&mut self, ppu: &Ppu) -> State {
        if self.clock == 0 {
            self.clock = 1;
            return State::Tile;
        }
        self.clock = 0;
        self.attrs = Attributes::default();
        let lcdc = LCDC(ppu.regs.lcdc.read());
        let ly = ppu.regs.ly.read();
        let scx = ppu.regs.scx.read();
        let scy = ppu.regs.scy.read();
        let (offset, y, x) = match (self.window_active(), lcdc.bg_area(), lcdc.win_area()) {
            (false, n , _) => (n, ly.wrapping_add(scy) as u16 / 8, (self.x + scx / 8) & 0x1F), //bg tile (LCDC.3)
            (true, _, n) => (n, ppu.win.y as u16 / 8, self.x) //window tile (LCDC.6) TODO INVESTIGATE THIS, IT STINKS (self.x)
        };
        let addr = 0x1800 | (offset as u16) << 10 | y << 5 as u16 | x as u16;
        self.tile = ppu.vram().get(Source::Ppu, |vram| vram.read_bank(addr, 0)) as u16;
        if ppu.regs.key0.value() & CGB_MODE != 0 {
            self.attrs = Attributes(ppu.vram().get(Source::Ppu, |v| v.read_bank(addr, 1)));
        }
        State::DataLow
    }

    fn get_tile_data(&self, ppu: &Ppu, high: bool) -> u8 {
        let lcdc = ppu.lcdc;
        let scy = ppu.regs.scy.read();
        let ly = ppu.regs.ly.read();
        let (wrap, tile) = if let Mode::Sprite(..) = self.mode {
            if lcdc.obj_tall() { (0xF, self.tile & 0xFE)  } else { (0x7, self.tile) }
        } else { (0x7, self.tile) };
        let y = (match self.mode {
            Mode::Bg => ly.wrapping_add(scy),
            Mode::Window => ppu.win.y,
            Mode::Sprite(s, ..) => ly.wrapping_sub(s.y),
        } & wrap) as u16;
        let y = if self.attrs.flip_y() { wrap as u16 - y } else { y };
        let addr = (match self.mode {
            Mode::Bg | Mode::Window => !(!lcdc.relative_addr() || (tile & 0x80) != 0) as u16,
            Mode::Sprite( .. ) => 0
        } << 12) | (tile << 4) | (y << 1) | (high as u16);
        ppu.vram().get(Source::Ppu, |v| v.read_bank(addr, self.attrs.bank()))
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

    fn data_high(&mut self, ppu: &Ppu, fifo: &mut BgFifo, oam: &mut ObjFifo) -> State {
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

    fn push(&mut self, ppu: &Ppu, bg: &mut BgFifo, oam: &mut ObjFifo) -> State {
        if let (Some(low), Some(high)) = (self.low.take(), self.high.take()) {
            let mut colors = [0; 8];
            colors.iter_mut().enumerate().for_each(|(i, c)| {
                let x = 7 - i;
                *c = ((low >> x) & 0x1) | (((high >> x) & 0x1) << 1);
            });
            if self.attrs.flip_x() { colors.reverse(); }
            if let Mode::Sprite(sp, n) = self.mode {
                let s = 8u8.saturating_sub(sp.x) as usize;
                colors.rotate_left(s);
                colors[8-s..].iter_mut().for_each(|x| *x = 0);
                oam.merge(colors.into_iter().map(|x| Pixel::sprite(x, n, self.attrs)).collect());
                self.set_mode(self.prev);
                bg.enable();
            } else if bg.push(colors.into_iter().map(|x|
                Pixel::bg(if ppu.regs.key0.value() & CGB_MODE == 0 && !ppu.lcdc.priority() { 0 } else { x }, self.attrs)
            ).collect()) {
                self.x += 1;
            } else {
                self.low = Some(low);
                self.high = Some(high);
                return State::Push
            }
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

    pub(crate) fn tick(&mut self, ppu: &Ppu, bg: &mut BgFifo, oam: &mut ObjFifo) {
        self.state = match self.state {
            State::Tile => self.get_tile(ppu),
            State::DataLow => self.data_low(ppu),
            State::DataHigh => self.data_high(ppu, bg, oam),
            State::Sleep => self.sleep(ppu),
            State::Push => self.push(ppu, bg, oam),
        };
    }
}
