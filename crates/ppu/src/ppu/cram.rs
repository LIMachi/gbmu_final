use shared::io::{CGB_MODE, IO, IODevice, IORegs};
use shared::mem::IOBus;

pub struct CRAM {
    bgdata: [u8; 64],
    objdata: [u8; 64],
}

impl Default for CRAM {
    fn default() -> Self {
        Self {
            bgdata: [0xFF; 64],
            objdata: [0; 64],
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
    pub fn color(&self, pixel: super::Pixel, io: &mut IORegs) -> [u8; 3] {
        match (pixel.color, pixel.attrs, pixel.sprite, io.io(IO::KEY0).value() & CGB_MODE != 0) {
            (c, a, true, false) => {
                let palette = if a.obp1() { io.io(IO::OBP1) } else { io.io_mut(IO::OBP0) }.read() >> (2 * c);
                io.palette().color(palette & 3)
            }
            (c, _, false, false) => {
                let palette = io.io(IO::BGP).read() >> (2 * c);
                io.palette().color(palette & 3)
            }
            (c, a, true, true) => {
                let palette = a.palette();
                let rgb555 = self.objdata[palette * 8 + c as usize * 2] as u16 | (self.objdata[palette * 8 + c as usize * 2 + 1] as u16) << 8;
                rgb555.to_bytes()
            }
            (c, a, false, true) => {
                let palette = a.palette();
                let rgb555 = self.bgdata[palette * 8 + c as usize * 2] as u16 | (self.bgdata[palette * 8 + c as usize * 2 + 1] as u16) << 8;
                rgb555.to_bytes()
            }
        }
    }
}

impl IODevice for CRAM {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        match io {
            IO::BCPD => {
                let bcps = bus.io_mut(IO::BCPS);
                let inc = bcps.bit(7) != 0;
                let addr = bcps.value() & 0x3F;
                if inc { bcps.direct_write(0x80 | ((addr + 1) & 0x3F)); }
                self.bgdata[addr as usize] = v;
            }
            IO::OCPD => {
                let ocps = bus.io_mut(IO::OCPS);
                let inc = ocps.bit(7) != 0;
                let addr = ocps.value() & 0x3F;
                if inc { ocps.direct_write(0x80 | ((addr + 1) & 0x3F)); }
                self.objdata[addr as usize] = v;
            }
            _ => {}
        }
    }
}
