use std::net::Ipv4Addr;
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use crate::com::Event;

mod com;

pub struct Port {
    int: IOReg,
    ctrl: IOReg,
    data: IOReg,
    ds: IOReg,
    cgb: IOReg,
    cable: com::Serial,
    transfer: Option<(u8, u8)>,
    recv: u8,
    clk: u32,
    dc: usize,
}

impl Port {
    pub fn new() -> Self {
        Self {
            ctrl: IOReg::unset(),
            data: IOReg::unset(),
            ds: IOReg::unset(),
            cgb: IOReg::unset(),
            int: IOReg::unset(),
            cable: com::Serial::build(),
            recv: 0,
            clk: 0,
            transfer: None,
            dc: 0,
        }
    }

    pub fn connect(&mut self, addr: Ipv4Addr, port: u16) {
        log::info!("trying to connect to remote...");
        self.cable.connect(addr, port);
    }

    pub fn tick(&mut self) {
        match self.cable.event() {
            Some(Event::Connected(addr)) => {
                log::info!("connected to peer {addr:?}");
                self.clk = 0;
                self.dc = 0;
                self.transfer = None;
            },
            _ => {}
        }
        if self.ctrl.dirty() {
            if self.ctrl.bit(0) != 0 {
                self.transfer = Some((0, 0));
                self.clk = 0;
                self.recv = 0;
            }
            self.ctrl.reset_dirty();
        }
        if let Some((i, o)) = {
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
                    self.clk += if self.ds.bit(7) != 0 { 2 } else { 1 };
                    let h = if self.ctrl.bit(1) != 0 { 16 } else { 512 };
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
        }
        {
            if i == 8 {  // received everything
                self.transfer = None;
                self.clk = 0;
                self.data.direct_write(self.recv);
                self.recv = 0;
                self.ctrl.reset(7);
                self.int.set(3);
            } else {
                self.transfer = Some((i, o));
            }
        }
    }
}

impl Device for Port {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ds = bus.io(IO::KEY1);
        self.data = bus.io(IO::SB);
        self.ctrl = bus.io(IO::SC);
        self.cgb = bus.io(IO::CGB);
        self.int = bus.io(IO::IF);
    }
}
