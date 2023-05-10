extern crate core;

use std::net::Ipv4Addr;

use shared::io::{IO, IODevice, IORegs};
use shared::mem::IOBus;
use shared::serde::{Deserialize, Serialize};

use crate::com::{Msg, Serial};

pub mod com;
mod protocol;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Idle,
    Transfer,
    Respond,
    Ack,
}

pub struct Port {
    pub cable: Serial,
    ready: bool,
    bits: u8,
    cycles: usize,
    data: Option<u8>,
    state: State,
}

impl Default for Port {
    fn default() -> Self { Port::new(Serial::phantom()) }
}

impl Port {

    pub fn new(cable: Serial) -> Self {
        Self { data: None, cable, bits: 0, cycles: 0, ready: false, state: State::Idle }
    }

    pub fn link(&mut self) -> &mut Serial { &mut self.cable }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        log::info!("trying to connect to remote...");
        self.cable.connect(addr, port);
    }

    fn interrupt(&mut self, io: &mut IORegs) {
        let v = self.data.take().unwrap();
        log::info!("transfer finished, received {v:#02X}");
        io.io_mut(IO::SB).direct_write(v);
        io.io_mut(IO::SC).reset(7);
        io.int_set(3);
    }

    pub fn tick(&mut self, io: &mut IORegs) {
        let sc = io.io(IO::SC);
        if sc.value() & 0x81 == 0x81 {
            self.cycles += if sc.bit(1) == 0 { 1 } else { 32 };
            if self.cycles >= 128 {
                self.cycles -= 128;
                if !self.cable.connected() {
                    let sb = io.io_mut(IO::SB);
                    let v = sb.value();
                    sb.direct_write(v << 1 | 1);
                }
                log::info!("transfer bit {}", self.bits);
                self.bits += 1;
                if self.bits >= 8 {
                    if !self.cable.connected() {
                        self.state = State::Idle;
                        self.data = Some(io.io(IO::SB).value());
                        self.interrupt(io);
                    } else if let (State::Transfer, Some(o)) = (self.state, self.data) {
                        self.state = State::Ack;
                        self.cable.send(Msg::Ack);
                    } else {
                        log::info!("transfer too long, stalling ({} bits cycles passed)", self.bits - 8);
                    }
                }
            }
            if self.state == State::Idle {
                self.state = State::Transfer;
                let o = io.io(IO::SB).value();
                log::info!("sending {o:#02X}");
                self.cable.send(Msg::Transfer(o));
            }
        }
        match (self.state, self.cable.recv()) {
            (State::Idle, Some(Msg::Transfer(o))) => {
                self.state = State::Respond;
                self.data = Some(o);
                let o = io.io(IO::SB).value();
                log::info!("responding {o:#02X}");
                self.cable.send(Msg::Respond(o));
            }
            (State::Transfer, Some(Msg::Respond(o))) => {
                self.data = Some(o);
            }
            (State::Respond, Some(Msg::Ack)) => {
                self.state = State::Idle;
                self.interrupt(io);
                self.cable.send(Msg::Ack);
            }
            (State::Ack, Some(Msg::Ack)) => {
                self.state = State::Idle;
                self.interrupt(io);
            }
            (state, Some(m)) => log::warn!("received unexpected {m:?} while in {state:?} state"),
            (_, None) => {}
        }
    }

    pub fn disconnect(&mut self) -> Serial {
        std::mem::replace(&mut self.cable, Serial::phantom())
    }
}

impl IODevice for Port {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        if io == IO::SC && v & 0x81 == 0x81 {
            self.cycles = 0;
            self.bits = 0;
            log::info!("Requested transfer ({:#02X})", bus.io(IO::SB).value());
        }
        if io == IO::SB {
            log::info!("preparing byte for transfer {v:#02X}")
        }
    }
}
