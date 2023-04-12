use std::net::Ipv4Addr;
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use crate::com::{Event, Serial};

pub mod com;

pub struct Link {
    pub port: u16,
    cable: Option<Serial>,
}

impl Link {
    pub fn new() -> Self {
        let serial = Serial::build();
        let port = serial.port;
        Self {
            port,
            cable: Some(serial)
        }
    }

    pub fn port(&mut self) -> Serial {
        self.cable.take().unwrap()
    }

    pub fn as_ref(&self) -> Option<&Serial> { self.cable.as_ref() }
    pub fn as_mut(&mut self) -> Option<&mut Serial> { self.cable.as_mut() }

    /// Assumption: this is the port that was given out by Self::port()
    /// we're only retrieving it. We're not supposed to have multiple copies flying around
    pub fn store(&mut self, serial: Serial) {
        self.cable = Some(serial);
    }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        self.cable.as_mut().unwrap().connect(addr, port);
    }

    pub fn borrowed(&self) -> bool { self.cable.is_none() }
}

pub struct Port {
    int: IOReg,
    ctrl: IOReg,
    data: IOReg,
    cable: Serial,
}

impl Port {
    pub fn new(cable: Serial) -> Self {
        Self {
            ctrl: IOReg::unset(),
            data: IOReg::unset(),
            int: IOReg::unset(),
            cable,
        }
    }

    pub fn link(&mut self) -> &mut Serial { &mut self.cable }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        log::info!("trying to connect to remote...");
        self.cable.connect(addr, port);
    }

    pub fn tick(&mut self) {
        if self.ctrl.dirty() {
            if self.ctrl.value() & 0x81 == 0x81 {
                self.cable.send(self.data.value());
            }
            self.ctrl.reset_dirty();
        }
        if let Some(o) = self.cable.recv() {
            if self.ctrl.bit(0) == 0 {
                self.cable.send(self.data.value());
            }
            self.data.direct_write(o);
            self.ctrl.reset(7);
            self.int.set(3);
        }
        /*if let Some((i, o)) = {
            if self.ctrl.bit(0) != 0 {
                if let Some((i, o)) = self.transfer.take() {
                    let mut i = if let Some(b) = self.cable.recv() {
                        self.recv <<= 1;
                        self.recv |= b;
                        i + 1 } else { i };
                    if !self.cable.connected() {
                        if self.dc < 84 { self.dc += 1; }
                        if i < o {
                            let b = if self.dc == 84 { 1 } else { self.recv & 1 };
                            self.recv <<= 1;
                            self.recv |= b;
                            log::info!("(disconnected) master, received bit {i}({b:#01b})");
                            i += 1;
                        }
                    }
                    let h = if self.ctrl.bit(1) != 0 { 4 } else { 128 };
                    self.clk += 1;
                    Some((i, if o == 8 { o } else if self.clk >= h {
                        self.clk -= h;
                        let b = self.data.bit(7 - o);
                        self.cable.send(b);
                        o + 1
                    } else { o }))
                } else { None }
            } else if let Some(b) = self.cable.recv() {
                let (i, o) = self.transfer.unwrap_or((0, 0));
                self.recv <<= 1;
                self.recv |= b;
                self.cable.send(self.data.bit(7 - o));
                Some((i + 1, o + 1))
            } else { self.transfer }
        } {
            if i == 8 {  // received everything
                self.transfer = None;
                self.clk = 0;
                self.data.direct_write(self.recv);
                self.recv = 0;
                self.ctrl.reset(7);
                self.int.set(3);
                log::info!("requested serial interrupt");
            } else {
                self.transfer = Some((i, o));
            }
        }*/
    }

    pub fn disconnect(&mut self) -> Serial {
        std::mem::replace(&mut self.cable, Serial::phantom())
    }
}

impl Device for Port {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.data = bus.io(IO::SB);
        self.ctrl = bus.io(IO::SC);
        self.int = bus.io(IO::IF);
    }
}
