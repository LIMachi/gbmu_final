use std::net::Ipv4Addr;

use shared::io::{IO, IODevice, IORegs};
use shared::mem::IOBus;

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
            cable: Some(serial),
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
    ready: bool,
    bits: u8,
    cycles: usize,
    data: Option<u8>,
}

impl Default for Port {
    fn default() -> Self { Port::new(Serial::phantom()) }
}

impl Port {
    pub fn new(cable: Serial) -> Self {
        Self { data: None, cable, bits: 0, cycles: 0, ready: false }
    }

    pub fn link(&mut self) -> &mut Serial { &mut self.cable }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        log::info!("trying to connect to remote...");
        self.cable.connect(addr, port);
    }

    pub fn tick(&mut self, io: &mut IORegs) {
        if !self.cable.connected() && io.io(IO::SC).value() & 0x80 == 0x80 {
            let sb = io.io_mut(IO::SB);
            let v = sb.value();
            if v != 0xFF {
                let v = v << 1 | 1;
                sb.direct_write(v);
                if v == 0xFF {
                    io.io_mut(IO::SC).reset(7);
                    io.int_set(3);
                }
            }
        }

        if let Some(o) = self.cable.recv() {
            log::info!("recv {o:#02X}");
            self.data = Some(o);
            if io.io(IO::SC).bit(0) == 0 {
                let v = io.io(IO::SB).value();
                log::info!("sending back {v:#02X}");
                self.cable.send(v);
            }
            self.bits = 8;
            self.cycles = 0;
        }
        if self.bits > 0 {
            self.cycles += 1;
            if self.cycles == 128 {
                self.cycles = 0;
                self.bits -= 1;
                if self.bits == 0 { self.ready = true; }
            }
        }
        if self.data.is_some() && self.ready {
            log::info!("serial interrupt");
            self.ready = false;
            io.io_mut(IO::SC).reset(7);
            io.io_mut(IO::SB).direct_write(self.data.take().unwrap());
            io.int_set(3);
        }
    }

    pub fn disconnect(&mut self) -> Serial {
        std::mem::replace(&mut self.cable, Serial::phantom())
    }
}

impl IODevice for Port {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        if io == IO::SC && v & 0x81 == 0x81 {
            let v = bus.io(IO::SB).value();
            log::info!("sending {v:#02X}");
            self.cable.send(v);
            self.bits = 8;
            self.cycles = 0;
        }
    }
}
