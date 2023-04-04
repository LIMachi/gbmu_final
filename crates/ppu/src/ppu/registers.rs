use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

#[derive(Default)]
pub struct Registers {
    pub lcdc: IOReg,
    pub stat: IOReg,
    pub scy: IOReg,
    pub scx: IOReg,
    pub ly: IOReg,
    pub lyc: IOReg,
    pub bgp: IOReg,
    pub obp0: IOReg,
    pub obp1: IOReg,
    pub wy: IOReg,
    pub wx: IOReg,
    pub bcps: IOReg,
    pub bcpd: IOReg,
    pub ocps: IOReg,
    pub ocpd: IOReg,
    pub opri: IOReg,
    pub interrupt: IOReg,
    pub cgb: IOReg,
    pub key0: IOReg,
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
        self.wy = bus.io(IO::WY);
        self.bcps = bus.io(IO::BCPS);
        self.bcpd = bus.io(IO::BCPD);
        self.ocps = bus.io(IO::OCPS);
        self.ocpd = bus.io(IO::OCPD);
        self.opri = bus.io(IO::OPRI);
        self.interrupt = bus.io(IO::IF);
        self.cgb = bus.io(IO::CGB);
        self.key0 = bus.io(IO::KEY0);
    }
}
