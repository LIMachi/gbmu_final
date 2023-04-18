use std::time::{Duration, Instant};

pub struct Clock {
    inner: u8,
    round: u8
}

impl Clock {
    pub fn new(cycle_len: u8) -> Self {
        Self {
            inner: 0,
            round: cycle_len
        }
    }
    pub fn tick(&mut self) -> u8 {
        self.inner += 1;
        self.inner %= self.round;
        self.inner
    }

    pub fn value(&self) -> u8 { self.inner }
}

pub struct Chrono {
    el: Duration,
    st: Instant,
    on: bool
}

impl Chrono {
    pub fn new() -> Self {
        Self {
            el: Duration::default(),
            st: Instant::now(),
            on: false
        }
    }

    pub fn paused(&self) -> bool { self.on == false }

    pub fn start(&mut self) {
        self.st = Instant::now();
        self.on = true;
    }

    pub fn pause(&mut self) {
        if self.on {
            self.el += self.st.elapsed();
        }
        self.on = false;
    }

    pub fn elapsed(&mut self) -> Duration {
        if self.on {
            self.el += self.st.elapsed();
            self.st = Instant::now();
        }
        self.el
    }

    pub fn wrap(&mut self) {
        self.el -= Duration::new(self.el.as_secs(), 0);
    }

    pub fn stop(&mut self) {
        self.on = false;
        self.el = Duration::default();
        self.st = Instant::now();
    }

    pub fn restart(&mut self) {
        self.el = Duration::default();
        self.st = Instant::now();
        self.on = true;
    }
}
