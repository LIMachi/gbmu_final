use std::time::{Instant, SystemTime};

#[derive(Clone)]
pub struct Rtc {
    latch_s: u8,
    latch_m: u8,
    latch_h: u8,
    latch_dl: u8,
    latch_dh: u8,

    seconds: u64,
    st: Option<Instant>
}

impl Default for Rtc {
    fn default() -> Self {
        Self {
            latch_s: 0,
            latch_m: 0,
            latch_h: 0,
            latch_dl: 0,
            latch_dh: 0,
            seconds: 0,
            st: Some(Instant::now())
        }
    }
}

pub struct RtcSave {
    ls: u8,
    lm: u8,
    lh: u8,
    dl: u8,
    dh: u8,
    epoch: u64
}

impl RtcSave {
    pub fn save(rtc: &Rtc) -> Self {
        let mut rtc = rtc.clone();
        rtc.latch();
        let epoch = SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|x| x.as_secs())
            .unwrap_or(0);
        RtcSave {
            ls: rtc.latch_s,
            lm: rtc.latch_m,
            lh: rtc.latch_h,
            dl: rtc.latch_dl,
            dh: rtc.latch_dh,
            epoch
        }
    }

    pub fn load(self) -> Rtc {
        let elapsed = SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|x| x.as_secs()).unwrap_or(0).saturating_sub(self.epoch);
        let mut rtc = Rtc::default();
        rtc.write(0x8, self.ls);
        rtc.write(0x9, self.lm);
        rtc.write(0xA, self.lh);
        rtc.write(0xB, self.dl);
        rtc.write(0xC, self.dh);
        if self.dh & 0x40 == 0 { rtc.seconds += elapsed; rtc.st = Some(Instant::now()); }
        rtc
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut ser = vec![self.ls, self.lm, self.lh, self.dl, self.dh];
        ser.extend(self.epoch.to_le_bytes());
        ser
    }

    pub fn deserialize(raw: Vec<u8>) -> Option<Self> {
        let [ls, lm, lh, dl, dh] = raw[0..5] else { return None };
        let [e0, e1, e2, e3, e4, e5, e6, e7] = raw[5..13] else { return None };
        let epoch = u64::from_le_bytes([e0, e1, e2, e3, e4, e5, e6, e7]);
        Some(Self { ls, lm, lh, dl, dh, epoch })
    }
}

impl Rtc {

    pub fn read(&self, reg: u8) -> u8 {
        match reg {
            0x8 => self.latch_s,
            0x9 => self.latch_m,
            0xA => self.latch_h,
            0xB => self.latch_dl,
            0xC => self.latch_dh,
            _ => unreachable!()
        }
    }

    fn timestamp(&self) -> u64 {
        self.seconds + self.st.map(|x| x.elapsed().as_secs()).unwrap_or(0)
    }

    pub fn write(&mut self, reg: u8, value: u8) {
        match reg {
            0x8 => self.latch_s = value,
            0x9 => self.latch_m = value,
            0xA => self.latch_h = value,
            0xB => self.latch_dl = value,
            0xC => self.latch_dh = value,
            _ => unreachable!()
        }
        if self.st.is_some() { self.st = Some(Instant::now()) }
        self.seconds = self.latch_dl as u64;
        let h = self.latch_dh as u64;
        self.seconds |= (h & 1) << 8;
        self.seconds *= 86_400;
        self.seconds += (self.latch_h as u64) * 3600;
        self.seconds += (self.latch_m as u64) * 60;
        self.seconds += self.latch_s as u64;
    }

    pub fn latch(&mut self) {
        let ts = self.timestamp();
        let (mut d, r) = (ts / 86400, ts %  86_400);
        if d >= 512 {
            self.latch_dh |= 0x80;
            d %= 512;
            self.seconds = r + 86400 * d;
            if self.st.is_some() { self.st = Some(Instant::now()) }
        }
        self.latch_dl = d as u8 & 0xFF;
        self.latch_dh &= 0xC0;
        self.latch_dh |= ((d >> 8) & 0x1) as u8;
        let (h, r) = (r / 3600, r % 3600);
        self.latch_h = h as u8;
        let (m, s) = (r / 60, r % 60);
        self.latch_m = m as u8;
        self.latch_s = s as u8;
    }

    pub fn halt(&mut self, halt: bool) {
        self.latch_dh &= 0x81;
        self.latch_dh |= (halt as u8) << 6;
        match (halt, self.st.take()) {
            (true, Some(st)) => self.seconds += st.elapsed().as_secs(),
            (false, _) => self.st = Some(Instant::now()),
            _ => {}
        }
    }
}
