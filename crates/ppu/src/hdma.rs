use shared::io::{CGB_MODE, IOReg};
use shared::mem::{IOBus, Source};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Mode {
    General,
    HBlank
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Wait,
    Transfer,
    Active,
    WaitHblank
}

impl Mode {
    pub fn new(mode: bool) -> Mode {
        if mode { Mode::HBlank } else { Mode::General }
    }
}

pub struct Hdma {
    mode: Option<Mode>,
    state: State,
    src_high: IOReg, // FF51
    src_low: IOReg, // FF52
    dest_high: IOReg, // FF53
    dest_low: IOReg, // FF54
    control: IOReg, // FF55
    statv: u8,
    stat: IOReg,
    key0: IOReg,
    count: u8,
    src: u16,
    dst: u16,
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            key0: IOReg::unset(),
            stat: IOReg::unset(),
            src_high: IOReg::unset(),
            src_low: IOReg::unset(),
            dest_high: IOReg::unset(),
            dest_low: IOReg::unset(),
            control: IOReg::unset(),
            state: State::Wait,
            statv: 0,
            mode: None,
            count: 0,
            src: 0,
            dst: 0
        }
    }
}

impl Hdma {

    fn transfer(&mut self, bus: &mut dyn IOBus) -> bool {
        let v = bus.read_with(self.src, Source::Hdma);
        self.src += 1;
        bus.write_with(self.dst, v, Source::Hdma);
        self.dst += 1;
        if self.dst == 0xA000 {
            log::warn!("HDMA overflow");
            self.mode = None;
            self.state = State::Wait;
            true
        } else { false }
    }

    pub fn tick(&mut self, bus: &mut dyn IOBus) -> bool {
        if self.key0.value() & CGB_MODE == 0 { return false };
        if self.control.dirty() {
            self.control.reset_dirty();
            match self.mode {
                None => {
                    let ctrl = self.control.value();
                    let mode = Mode::new(ctrl & 0x80 != 0);
                    self.src = u16::from_le_bytes([self.src_low.value(), self.src_high.value()]) & 0xFFF0;
                    self.dst = (u16::from_le_bytes([self.dest_low.value(), self.dest_high.value()]) & 0x1FF0) + 0x8000;
                    let src = self.src;
                    let dst = self.dst;
                    let len = ((ctrl & 0x7F) as usize + 1) * 0x10;
                    self.statv = self.stat.value() & 0x3;
                    self.mode = Some(mode);
                    self.state = if mode == Mode::HBlank { State::WaitHblank } else { State::Transfer };
                    log::info!("HDMA ({mode:?}): [{src:#04X}-{:#04X}] -> [{dst:#04X}-{:#04X}] ({})", src + len as u16, dst + len as u16, len);
                },
                Some(_) if self.control.bit(7) == 0 => {
                    log::info!("HDMA paused");
                    self.control.set(7);
                    self.count = 0;
                    self.mode = None;
                },
                _ => unreachable!()
            }
        }
        let mut tick = false;
        self.state = match self.state {
            s@ (State::WaitHblank | State::Active) => {
                let old = self.statv;
                self.statv = self.stat.value() & 0x3;
                if old != 0 && self.statv == 0 {
                    log::info!("new batch");
                    tick = true;
                    State::Transfer
                } else { s }
            },
            State::Transfer if self.mode == Some(Mode::General) => {
                tick = true;
                if self.transfer(bus) { return false }
                self.count += 1;
                if self.count == 0x10 {
                    let (len, done) = (self.control.value() & 0x7F).overflowing_sub(1);
                    self.control.direct_write(len);
                    self.count = 0;
                    if done {
                        log::info!("General DMA complete.");
                        self.mode = None; State::Wait
                    } else { State::Transfer }
                } else { State::Transfer }
            },
            State::Transfer if self.mode == Some(Mode::HBlank) => {
                log::info!("{:04X} -> {:04X} ({}/16)", self.src, self.dst, self.count);
                tick = true;
                if self.transfer(bus) { return false }
                self.count += 1;
                if self.count == 0x10 {
                    let (len, done) = (self.control.value() & 0x7F).overflowing_sub(1);
                    self.control.direct_write(len | 0x80);
                    self.count = 0;
                    if done {
                        log::info!("HDMA complete.");
                        self.mode = None; State::Wait
                    } else { State::Active }
                } else { State::Transfer }
            },
            state => state
        };
        tick
    }
}
