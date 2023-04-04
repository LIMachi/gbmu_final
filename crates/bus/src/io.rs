use shared::io::{Access, AccessMode, DMG_MODE, IO, IOReg};
use shared::mem::Mem;


pub struct IORegs {
    cgb: IOReg,
    range: Vec<IOReg>
}

impl IORegs {
    const DISABLED: AccessMode = AccessMode::Generic(Access::U);

    pub fn init(cgb: bool) -> Self {
        Self {
            cgb: IOReg::rdonly().with_value(cgb as u8),
            range: (0..128).into_iter().map(|i| {
                let (access, value) = IO::try_from(0xFF00 + i)
                    .map(|x| (x.access(), x.default(cgb))).unwrap_or(Default::default());
                IOReg::with_access(access)
                    .with_value(value)
            }).collect()
        }
    }

    fn set(&mut self, io: IO, value: u8) {
        let addr = io as u16 - shared::mem::IO;
        self.range[addr as usize].direct_write(value);
    }

    pub fn compat_mode(&mut self) {
        if self.io(IO::KEY0).value() == DMG_MODE {
            log::info!("DMG compatibility mode: enabled");
            self.io(IO::HDMA5).set_access(IORegs::DISABLED);
            self.io(IO::KEY1).set_access(IORegs::DISABLED);
            self.io(IO::OCPD).set_access(IORegs::DISABLED);
            self.io(IO::BCPD).set_access(IORegs::DISABLED);
            self.io(IO::SVBK).set_access(IORegs::DISABLED);
            self.io(IO::VBK).set_access(IORegs::DISABLED);
            self.io(IO::OPRI).set_access(IORegs::DISABLED);
        }
    }

    pub fn post(&mut self) {
        log::info!("compat mode: {:#04X}", self.io(IO::KEY0).value());
        self.io(IO::POST).set_access(IORegs::DISABLED);
        self.io(IO::KEY0).set_access(IORegs::DISABLED);
    }

    pub fn skip_boot(&mut self, console: u8) {
        self.set(IO::KEY0, console);
        self.set(IO::BGP, 0xFC);
        self.set(IO::OBP0, 0xFF);
        self.set(IO::OBP1, 0xFF);
        self.set(IO::LCDC, 0x91);
        self.compat_mode();
        self.post();
    }

    pub fn io(&self, io: IO) -> IOReg {
        if io == IO::CGB { return self.cgb.clone() }
        self.range[io as u16 as usize - shared::mem::IO as usize].clone()
    }

    pub fn writable(&self, io: IO) -> bool {
        self.range[io as u16 as usize - shared::mem::IO as usize].writable()
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
