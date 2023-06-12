use serde::{Deserialize, Serialize};
use shared::io::{CGB_MODE, IO, IODevice, IORegs};
use shared::mem::IOBus;
use shared::utils::serde_arrays;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct CRAM {
    #[serde(with = "serde_arrays")]
    bgdata: [u8; 64],
    #[serde(with = "serde_arrays")]
    objdata: [u8; 64],
    dmgbgpal: [[u8; 3]; 4],
    dmgobj0pal: [[u8; 3]; 4],
    dmgobj1pal: [[u8; 3]; 4],
    posted_in_cgb: bool,
}

impl Default for CRAM {
    fn default() -> Self {
        Self {
            bgdata: [0xFF; 64],
            objdata: [0xFF; 64],
            dmgbgpal: [[0xFF; 3]; 4],
            dmgobj0pal: [[0xFF; 3]; 4],
            dmgobj1pal: [[0xFF; 3]; 4],
            posted_in_cgb: false,
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
    pub fn color(&self, pixel: super::Pixel, io: &IORegs) -> [u8; 3] {
        match (pixel.color, pixel.attrs, pixel.sprite, io.io(IO::KEY0).value() & CGB_MODE != 0) {
            (c, a, true, false) => {
                let palette = if a.obp1() { io.io(IO::OBP1) } else { io.io(IO::OBP0) }.read() >> (2 * c);
                if self.posted_in_cgb {
                    if a.obp1() {
                        self.dmgobj1pal[(palette & 3) as usize]
                    } else {
                        self.dmgobj0pal[(palette & 3) as usize]
                    }
                } else {
                    io.palette().color(palette & 3)
                }
            }
            (c, _, false, false) => {
                let palette = io.io(IO::BGP).read() >> (2 * c);
                if self.posted_in_cgb {
                    self.dmgbgpal[(palette & 3) as usize]
                } else {
                    io.palette().color(palette & 3)
                }
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

    fn palette_from_boot(&mut self) {
        for c in 0..4 {
            let rgb555 = self.bgdata[c * 2] as u16 | (self.bgdata[c * 2 + 1] as u16) << 8;
            self.dmgbgpal[c] = rgb555.to_bytes();
        }
        for o in 0..2 {
            for c in 0..4 {
                let rgb555 = self.objdata[o * 8 + c * 2] as u16 | (self.objdata[o * 8 + c * 2 + 1] as u16) << 8;
                if o == 0 {
                    self.dmgobj0pal[c] = rgb555.to_bytes();
                } else {
                    self.dmgobj1pal[c] = rgb555.to_bytes();
                }
            }
        }
        self.posted_in_cgb = true;
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
            IO::POST => {
                // self.dump();
                if bus.io(IO::CGB).value() != 0 {
                    self.palette_from_boot();
                }
            }
            _ => {}
        }
    }
}
