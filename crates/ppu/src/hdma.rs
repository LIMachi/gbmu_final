use shared::io::{CGB_MODE, IO, IODevice};
use shared::mem::{IOBus, Source};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Mode {
    General,
    HBlank,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Wait,
    Transfer,
    Active,
    WaitHblank,
}

impl Mode {
    pub fn new(mode: bool) -> Mode {
        if mode { Mode::HBlank } else { Mode::General }
    }
}

pub struct Hdma {
    mode: Option<Mode>,
    state: State,
    statv: u8,
    count: u8,
    src: u16,
    dst: u16,
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            state: State::Wait,
            statv: 0,
            mode: None,
            count: 0,
            src: 0,
            dst: 0,
        }
    }
}

impl Hdma {
    fn transfer(&mut self, bus: &mut dyn IOBus) -> bool {
        let v = bus.read_with(self.src, Source::Hdma);
        self.src += 1;
        log::info!("[{:#04X}] {:#02X} -> [{:#04X}]", self.src - 1, v, self.dst);
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
        if bus.io(IO::KEY0).value() & CGB_MODE == 0 { return false; };
        let mut tick = false;
        self.state = match self.state {
            s @ (State::WaitHblank | State::Active) => {
                let stat = bus.io(IO::STAT).value();
                let old = self.statv;
                self.statv = stat & 0x3;
                if old != 0 && self.statv == 0 {
                    tick = true;
                    State::Transfer
                } else { s }
            }
            State::Transfer if self.mode == Some(Mode::General) => {
                tick = true;
                if self.transfer(bus) { return false; }
                self.count += 1;
                if self.count == 0x10 {
                    let control = bus.io_mut(IO::HDMA5);
                    let (len, done) = (control.value() & 0x7F).overflowing_sub(1);
                    control.direct_write(len);
                    self.count = 0;
                    if done {
                        self.mode = None;
                        State::Wait
                    } else { State::Transfer }
                } else { State::Transfer }
            }
            State::Transfer if self.mode == Some(Mode::HBlank) => {
                tick = true;
                if self.transfer(bus) { return false; }
                self.count += 1;
                if self.count == 0x10 {
                    let control = bus.io_mut(IO::HDMA5);
                    let (len, done) = (control.value() & 0x7F).overflowing_sub(1);
                    control.direct_write(len | 0x80);
                    self.count = 0;
                    if done {
                        self.mode = None;
                        State::Wait
                    } else { State::Active }
                } else { State::Transfer }
            }
            state => state
        };
        tick
    }
}

impl IODevice for Hdma {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        if io == IO::HDMA5 {
            match self.mode {
                None => {
                    let mode = Mode::new(v & 0x80 != 0);
                    self.src = u16::from_le_bytes([bus.io(IO::HDMA2).value(), bus.io(IO::HDMA1).value()]) & 0xFFF0;
                    self.dst = (u16::from_le_bytes([bus.io(IO::HDMA4).value(), bus.io(IO::HDMA3).value()]) & 0x1FF0) + 0x8000;
                    log::info!("HDMA start ({mode:?}) {:#04X} -> {:#04X} ({})", self.src, self.dst, ((v & 0x7F) + 1) as u16 * 16);
                    self.statv = bus.io(IO::STAT).value() & 0x3;
                    self.mode = Some(mode);
                    self.state = if mode == Mode::HBlank { State::WaitHblank } else { State::Transfer };
                }
                Some(_) if v & 0x80 == 0 => {
                    bus.io_mut(IO::HDMA5).set(7);
                    self.count = 0;
                    self.mode = None;
                }
                _ => unreachable!()
            }
        }
    }
}
