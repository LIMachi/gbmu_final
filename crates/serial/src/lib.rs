use std::net::Ipv4Addr;
use shared::io::{IO, IORegs};
use crate::com::Serial;

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
    cable: Serial,
}

impl Default for Port {
    fn default() -> Self { Port::new(Serial::phantom()) }
}

impl Port {
    pub fn new(cable: Serial) -> Self {
        Self { cable }
    }

    pub fn link(&mut self) -> &mut Serial { &mut self.cable }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        log::info!("trying to connect to remote...");
        self.cable.connect(addr, port);
    }

    pub fn tick(&mut self, io: &mut IORegs) {
        let data = io.io_mut(IO::SB).value();
        let ctrl = io.io_mut(IO::SC);
        if ctrl.dirty() {
            if ctrl.value() & 0x81 == 0x81 {
                self.cable.send(data);
            }
            ctrl.reset_dirty();
        }
        if let Some(o) = self.cable.recv() {
            if ctrl.bit(0) == 0 {
                self.cable.send(data);
            }
            ctrl.reset(7);
            io.io_mut(IO::SB).direct_write(o);
            io.int_set(3);
        }
    }

    pub fn disconnect(&mut self) -> Serial {
        std::mem::replace(&mut self.cable, Serial::phantom())
    }
}
