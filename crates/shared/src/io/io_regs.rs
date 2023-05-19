use crate::utils::palette::Palette;

use super::*;

pub struct IORegs {
    palette: Palette,
    cgb: IOReg,
    range: Vec<IOReg>,
}

impl IORegs {
    const DISABLED: AccessMode = AccessMode::Generic(Access::U);

    pub fn init(cgb: bool, palette: Palette) -> Self {
        Self {
            palette,
            cgb: IOReg::rdonly().with_value(cgb as u8),
            range: (0..128).into_iter().map(|i| {
                let (access, value) = IO::try_from(0xFF00 + i)
                    .map(|x| (x.access(), x.default(cgb))).unwrap_or(Default::default());
                IOReg::with_access(access)
                    .with_value(value)
            }).collect(),
        }
    }

    fn set(&mut self, io: IO, value: u8) {
        let addr = io as u16 - crate::mem::IO;
        self.range[addr as usize].direct_write(value);
    }

    pub fn compat_mode(&mut self) {
        if self.io(IO::KEY0).value() == DMG_MODE {
            self.io_mut(IO::HDMA5).direct_write(0).set_access(IORegs::DISABLED);
            self.io_mut(IO::KEY1).direct_write(0).set_access(IORegs::DISABLED);
            self.io_mut(IO::OCPD).direct_write(0).set_access(IORegs::DISABLED);
            self.io_mut(IO::BCPD).direct_write(0).set_access(IORegs::DISABLED);
            self.io_mut(IO::SVBK).direct_write(0).set_access(IORegs::DISABLED);
            self.io_mut(IO::VBK).direct_write(0).set_access(IORegs::DISABLED);
        }
    }

    pub fn post(&mut self) {
        self.io_mut(IO::POST).set_access(IORegs::DISABLED);
        self.io_mut(IO::KEY0).set_access(IORegs::DISABLED);
        self.io_mut(IO::OPRI).set_access(IORegs::DISABLED);
    }

    pub fn skip_boot(&mut self, console: u8) {
        if console & 0x80 == 0 {
            self.set(IO::KEY0, DMG_MODE);
            self.set(IO::OPRI, 0x1);
        }
        self.set(IO::POST, 0x1);
        self.set(IO::BGP, 0xFC);
        self.set(IO::OBP0, 0xFF);
        self.set(IO::OBP1, 0xFF);
        self.set(IO::LCDC, 0x91);
        self.set(IO::STAT, 0x1);
        self.set(IO::DIV, 0xAC);
        self.compat_mode();
        self.post();
    }

    pub fn io(&self, io: IO) -> &IOReg {
        if io == IO::CGB { return &self.cgb; }
        &self.range[io as u16 as usize - crate::mem::IO as usize]
    }

    pub fn io_mut(&mut self, io: IO) -> &mut IOReg {
        if io == IO::CGB { return &mut self.cgb; }
        &mut self.range[io as u16 as usize - crate::mem::IO as usize]
    }

    pub fn io_addr(&mut self, io: u16) -> Option<&mut IOReg> {
        self.range.get_mut(io as usize - crate::mem::IO as usize)
    }

    pub fn int_set(&mut self, bit: u8) {
        self.io_mut(IO::IF).set(bit);
    }

    pub fn int_reset(&mut self, bit: u8) {
        self.io_mut(IO::IF).reset(bit);
    }

    pub fn writable(&self, io: IO) -> bool {
        self.range[io as u16 as usize - crate::mem::IO as usize].writable()
    }

    pub fn palette(&self) -> Palette { self.palette }
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }
}

impl Mem for IORegs {
    fn read(&self, addr: u16, _absolute: u16) -> u8 {
        self.range.get(addr as usize).map(|x| x.read()).expect(format!("read outside of IOReg range {addr:#06X}").as_str())
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.range.get_mut(addr as usize)
            .map(|x| x.write(0, value, absolute)).
            expect(format!("write outside of IOReg range {addr:#06X}").as_str());
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let end = ((st + len) as usize).min(self.range.len());
        let st = (st as usize).min(self.range.len() - 1);
        self.range[st..end].iter().map(|x| x.value()).collect()
    }
}
