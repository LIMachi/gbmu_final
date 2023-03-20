use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

pub struct CRAM {
    obp0: IOReg,
    obp1: IOReg,
    bgp: IOReg,
    bcps: IOReg,
    bcpd: IOReg,
    ocps: IOReg,
    ocpd: IOReg,
    bgdata: [u8; 64],
    objdata: [u8; 64],
    pub cgb: IOReg,
}

impl Default for CRAM {
    fn default() -> Self {
        Self {
            obp0: Default::default(),
            obp1: Default::default(),
            bgp: Default::default(),
            bcps: Default::default(),
            bcpd: Default::default(),
            ocps: Default::default(),
            ocpd: Default::default(),
            bgdata: [0xFF; 64],
            objdata: [0; 64],
            cgb: Default::default()
        }
    }
}

trait Rgb555 {
    fn to_bytes(self) -> [u8; 3];
}

impl Rgb555 for u16 {
    fn to_bytes(self) -> [u8; 3] {
        let r = (self & 0x1F) as u8;
        let g = (self >> 5) as u8 & 0x1F;
        let b = (self >> 10) as u8 & 0x1F;
        [r << 3 | r >> 2, g << 3 | g >> 2, b << 3 | b >> 2]
    }
}

impl CRAM {
    pub fn color(&self, pixel: super::Pixel) -> [u8; 3] {
        const DMG_COLORS: [[u8; 3]; 4] = [[0xBF; 3], [0x7F; 3], [0x3F; 3], [0; 3]];

        match (pixel.color, pixel.attrs, pixel.sprite, self.cgb.read() != 0) {
            (c, a, true, false) => {
                let palette = if a.obp1() { &self.obp1 } else { &self.obp0 }.read() >> (2 * c);
                DMG_COLORS[(palette & 3) as usize]
            },
            (c, _, false, false) => {
                let palette = self.bgp.read() >> (2 * c);
                DMG_COLORS[(palette & 3) as usize]
            },
            (c, a, true, true) => {
                let palette = a.palette();
                let rgb555 = self.objdata[palette * 8 + c as usize * 2] as u16 | (self.objdata[palette * 8 + c as usize * 2 + 1] as u16) << 8;
                rgb555.to_bytes()
            },
            (c, a, false, true) => {
                let palette = a.palette();
                let rgb555 = self.bgdata[palette * 8 + c as usize * 2] as u16 | (self.bgdata[palette * 8 + c as usize * 2 + 1] as u16) << 8;
                rgb555.to_bytes()
            },
        }
    }

    // TODO lock access
    pub fn tick(&mut self) {
        if self.bcpd.dirty() {
            //println!("{:#06X}-> {:04X}", self.bcps.value(), self.bcpd.value());
            self.bcpd.reset_dirty();
            let inc = self.bcps.bit(7) != 0;
            let addr = self.bcps.value() & 0x3F;
            if inc { self.bcps.direct_write(0x80 | ((addr + 1) & 0x3F)) }
            self.bgdata[addr as usize] = self.bcpd.value();
        }
        if self.ocpd.dirty() {
            self.ocpd.reset_dirty();
            let inc = self.ocps.bit(7) != 0;
            let addr = self.ocps.value() & 0x3F;
            if inc { self.ocps.direct_write(0x80 | ((addr + 1) & 0x3F)) }
            self.objdata[addr as usize] = self.ocpd.value();
        }
    }
}

impl Device for CRAM {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.obp0 = bus.io(IO::OBP0);
        self.obp1 = bus.io(IO::OBP1);
        self.bgp = bus.io(IO::BGP);
        self.bcps = bus.io(IO::BCPS);
        self.bcpd = bus.io(IO::BCPD);
        self.ocps = bus.io(IO::OCPS);
        self.ocpd = bus.io(IO::OCPD);
        self.cgb = bus.io(IO::CGB);
    }
}
