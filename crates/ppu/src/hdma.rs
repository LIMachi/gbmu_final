use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, Source};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Mode {
    General,
    HBlank
}

impl Mode {
    pub fn new(mode: bool) -> Mode {
        if mode { Mode::HBlank } else { Mode::General }
    }
}

pub struct Transfer {
    mode: Mode,
    len: usize,
    src: u16,
    dst: u16
}

impl Transfer {
    fn new(mode: Mode, len: usize, st: u16, dst: u16) -> Self {
        Self {
            mode,
            len,
            src: st,
            dst
        }
    }

    pub fn should_tick(&self, stat: u8, blocks: usize) -> bool {
        if self.mode == Mode::General { true }
        else if stat != 0 { false }
        else {
            let mut bl = self.len / 16;
            if self.len & 0xF != 0 { bl += 1 }
            bl != blocks
        }
    }
}

pub struct Hdma {
    src_high: IOReg, // FF51
    src_low: IOReg, // FF52
    dest_high: IOReg, // FF53
    dest_low: IOReg, // FF54
    control: IOReg, // FF55
    stat: IOReg,
    cgb: IOReg,
    transfer: Option<Transfer>
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            transfer: None,
            cgb: IOReg::unset(),
            stat: IOReg::unset(),
            src_high: IOReg::unset(),
            src_low: IOReg::unset(),
            dest_high: IOReg::unset(),
            dest_low: IOReg::unset(),
            control: IOReg::unset(),
        }
    }
}

impl Hdma {
    pub fn tick(&mut self, bus: &mut dyn IOBus) -> bool {
        if self.cgb.read() == 0 { return false };
        if self.control.dirty() {
            log::info!("HDMA started");
            self.control.reset_dirty();
            match self.transfer {
                None => {
                    let mode = Mode::new(self.control.bit(7) != 0);
                    let st  = u16::from_le_bytes([self.src_low.value(), self.src_high.value()]) & 0xFFF0;
                    let dst  = (u16::from_le_bytes([self.dest_low.value(), self.dest_high.value()]) & 0x1FF0) + 0x8000;
                    let len = (self.control.value() & 0x7F) as usize;
                    self.transfer = Some(Transfer::new(mode, (len + 1) * 0x10, st, dst));
                },
                Some( ..) if self.control.bit(7) == 0 => {
                    self.control.set(7);
                    self.transfer = None;
                },
                _ => unreachable!()
            }
        }
        let mut tick = false;
        self.transfer = if let Some(mut tr) = self.transfer.take() {
            let mut blocks = self.control.value();
            if tr.should_tick(self.stat.value() & 0x3, (self.control.value() as usize) & 0x7F) {
                tick = true;
                tr.len -= 1;
                let v = bus.read_with(tr.src, Source::Hdma);
                log::info!("cp {:#06X}({:#02X}) to {:#06X}", tr.src, v, tr.dst);
                bus.write_with(tr.dst, v, Source::Hdma);
                tr.src += 1;
                tr.dst += 1;
                blocks = (tr.len / 16).wrapping_sub(1) as u8 | 0x80;
                self.control.direct_write(blocks);
            }
            if blocks == 0xFF {
                None
            } else {
                Some(tr)
            }
        } else { None };
        tick
    }
}

impl Device for Hdma {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.stat = bus.io(IO::STAT);
        self.cgb = bus.io(IO::CGB);
        self.src_high = bus.io(IO::HDMA1);
        self.src_low = bus.io(IO::HDMA2);
        self.dest_high = bus.io(IO::HDMA3);
        self.dest_low = bus.io(IO::HDMA4);
        self.control = bus.io(IO::HDMA5);
    }
}
