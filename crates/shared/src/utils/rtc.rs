use std::time::{Instant, SystemTime};

#[derive(Clone, Debug)]
pub struct Rtc {
    quartz: u16,
    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
    ls: u8,
    lm: u8,
    lh: u8,
    ldl: u8,
    ldh: u8
}

impl Default for Rtc {
    fn default() -> Self {
        Self {
            quartz: 0,
            s: 0,
            m: 0,
            h: 0,
            dl: 0,
            dh: 0,
            ls: 0,
            lm: 0,
            lh: 0,
            ldl: 0,
            ldh: 0
        }
    }
}

impl Rtc {
    const MAX_SECONDS: u64 = 44_236_800;

    pub fn read(&self, reg: u8) -> u8 {
        match reg {
            0x8 => self.ls,
            0x9 => self.lm,
            0xA => self.lh,
            0xB => self.ldl,
            0xC => self.ldh,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, reg: u8, value: u8) {
        match reg {
            0x8 => self.s = value,
            0x9 => self.m = value,
            0xA => self.h = value,
            0xB => self.dl = value,
            0xC => self.dh = value,
            _ => unreachable!()
        }
        self.latch();
    }

    pub fn latch(&mut self) {
        self.ls = self.s & 0x3F;
        self.lm = self.m & 0x3F;
        self.lh = self.h & 0x1F;
        self.ldl = self.dl;
        self.ldh = self.dh & 0xC1;
    }

    pub fn tick(&mut self) {
        static mut LAST: Option<Instant> = None;
        if self.quartz == 32767 { self.quartz = 0; }
        else { self.quartz += 1; return ; }
        if self.dh & 0x40 != 0 { return };
        unsafe {
            if LAST.is_none() { LAST = Some(Instant::now()); }
            else {
                log::info!("time since last whole second tick: {:?}", LAST.unwrap().elapsed());
            }
        }
        self.s = (self.s + 1) & 0x3F;
        if self.s != 59 { return ; }
        self.m = (self.m + 1) & 0x3F;
        if self.m != 59 { return ; }
        self.h = (self.h + 1) & 0x1F;
        if self.h != 23 { return ; }
        let (dl, c) = self.dl.overflowing_add(1);
        if c {
            if self.dh & 1 != 0 { self.dh |= 0x80; }
            self.dh = (self.dh + 1) & 0xC1;
        }
        self.dl = dl;
    }

    pub fn halt(&mut self, halt: bool) {
        self.dh &= 0x81;
        if halt { self.dh |= 0x40 }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let epoch = SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|x| x.as_secs())
            .unwrap_or(0);
        let mut ser = vec![self.s, self.m, self.h, self.dl, self.dh, self.ls, self.lm, self.lh, self.ldl, self.ldh];
        ser.extend(epoch.to_le_bytes());
        ser
    }

    pub fn deserialize(raw: Vec<u8>) -> Option<Self> {
        if raw.len() != 18 { return None; }
        let [s, m, h, dl, dh] = raw[0..5] else { unreachable!()  };
        let [latch_s, latch_m, latch_h, latch_dl, latch_dh] = raw[5..10] else { unreachable!()  };
        let [e0, e1, e2, e3, e4, e5, e6, e7] = raw[10..18] else { unreachable!() };
        let epoch = u64::from_le_bytes([e0, e1, e2, e3, e4, e5, e6, e7]);
        let mut elapsed = SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|x| x.as_secs()).unwrap_or(0).saturating_sub(epoch);
        let mut rtc = Self { quartz: 0, s, m, h, dl, dh, ls: latch_s, lm: latch_m, lh: latch_h, ldl: latch_dl, ldh: latch_dh };
        if elapsed / Rtc::MAX_SECONDS > 0 { rtc.dh |= 0x80; }
        elapsed %= Rtc::MAX_SECONDS;
        if dh & 0x40 == 0 { for _ in 0..elapsed { rtc.tick(); } }
        Some(rtc)
    }
}
