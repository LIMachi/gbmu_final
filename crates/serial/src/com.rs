use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::time::Duration;

pub enum Event { Stop, Connected(SocketAddr), Disconnect }

struct Client {
    inner: TcpStream,
    data: Sender<u8>,
    done: Sender<()>,
    connected: Arc<AtomicBool>,
}

impl Client {
    pub fn spawn(client: TcpStream, data: Sender<u8>, done: Sender<()>, connected: Arc<AtomicBool>) -> Self {
        Self { inner: client, data, done, connected }
    }
    pub fn run(mut self) {
        std::thread::spawn(move || {
            self.inner.set_nonblocking(false).expect("block");
            let mut buf = [0; 1];
            loop {
                match self.inner.read_exact(&mut buf) {
                    Ok(()) => match self.data.send(buf[0]) {
                        Err(_) => { log::warn!("client: failed to send down recv data"); break },
                        _ => {}
                    },
                    Err(e) => {
                        match e.kind() {
                            ErrorKind::WouldBlock => {},
                            _ => break
                        }
                    }
                }
            }
            log::info!("Connection reset by peer");
            self.connected.store(false, Ordering::Relaxed);
            self.done.send(()).ok();
        });
    }
}


pub struct Serial {
    signal: Sender<Event>,
    events: Receiver<Event>,
    connect: Sender<(Ipv4Addr, u16)>,
    recv: Receiver<u8>,
    send: Sender<u8>,
    connected: Arc<AtomicBool>,
    pub(crate) port: u16,
}

pub struct Server {
    socket: TcpListener,
    connected: Arc<AtomicBool>,
    client: Option<TcpStream>,
    signal: Option<Receiver<()>>,
    data: Option<Receiver<u8>>,
    recv: Receiver<u8>,
    send: Sender<u8>,
    events: Sender<Event>,
    connect: Receiver<(Ipv4Addr, u16)>,
    stop: Receiver<Event>
}

impl Server {
    pub fn connect(&mut self, stream: TcpStream, addr: SocketAddr) {
        let (tx, rec_x) = channel();
        let (td, rec_d) = channel();
        self.data = Some(rec_x);
        self.client.as_ref().map(|x| x.shutdown(Shutdown::Both).ok());
        self.client = Some(stream.try_clone().expect("failed to clone socket"));
        log::info!("client connected from {addr:?}");
        if let Some(rd) = self.signal.as_mut() {
            while let Ok(_) = rd.recv_timeout(Duration::from_secs(5)) { }
        }
        self.signal = Some(rec_d);
        Client::spawn(stream, tx,  td, self.connected.clone()).run();
        self.connected.store(true, Ordering::Relaxed);
        self.events.send(Event::Connected(addr)).ok();
    }

    fn recv(&mut self) {
        if let Some(v) = match self.data.as_mut().map(|x| x.try_recv()) {
            Some(Ok(v)) => Some(v),
            Some(Err(TryRecvError::Disconnected)) => {
                self.data = None;
                self.client.take().map(|x| x.shutdown(Shutdown::Both));
                None
            },
            _ => None
        } { self.send.send(v).ok(); }
    }

    fn send(&mut self, data: u8) {
        match self.client.as_mut().map(|x| x.write(&[data])) {
            Some(Ok(0)) => { self.client = None; self.data = None; },
            Some(Err(e)) => { log::warn!("error client send: {e:?}") },
            _ => {}
        }
    }

    fn recv_events(&mut self) -> bool {
        match self.stop.try_recv().ok() {
            Some(Event::Stop) => true,
            Some(Event::Disconnect) => {
                self.client.as_ref().map(|x| x.shutdown(Shutdown::Both).ok());
                if let Some(rd) = self.signal.as_mut() {
                    while let Ok(_) = rd.recv_timeout(Duration::from_secs(5)) { }
                }
                self.client = None;
                self.signal = None;
                false
            },
            _ => unreachable!()
        }
    }

    pub fn run(mut self) {
        loop {
            match self.socket.accept() {
                Ok((stream, addr)) => self.connect(stream, addr),
                Err(e) if e.kind() != ErrorKind::WouldBlock => log::warn!("connection refused {e:?}"),
                _ => {}
            };
            self.recv();
            if let Some(data) = self.recv.try_recv().ok() { self.send(data); }
            if let Some((addr, port)) = self.connect.try_recv().ok() {
                if let Ok(stream) = TcpStream::connect((addr, port)) {
                    self.connect(stream, SocketAddr::new(IpAddr::V4(addr), port));
                }
            }
            if self.recv_events() { break }
            if let Some(_) = self.stop.try_recv().ok() { break; }
        }
    }
}

impl Serial {
    pub fn phantom() -> Self {
        let (ts, _rs) = channel();
        let (_te, re) = channel();
        let (tc, _rc) = channel();
        let (to, _ro) = channel();
        let (_ti, ri) = channel();
        Self {
            port: 0,
            signal: ts,
            events: re,
            connect: tc,
            recv: ri,
            send: to,
            connected: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn build() -> Self {
        let connected = Arc::new(AtomicBool::new(false));
        let srv = TcpListener::bind(
            &(27542..27552).map(|port| SocketAddr::from(([0, 0, 0, 0], port)))
                .collect::<Vec<SocketAddr>>()[..]).expect("failed to find available port");
        srv.set_nonblocking(true).expect("failed to set nonblocking");
        let port = srv.local_addr().map(|x| x.port()).unwrap_or_default();
        let (tx_e, rx_e) = channel();
        let (tx_end, rx_end) = channel();
        let (tx_r, rx_r) = channel();
        let (tx_s, rx_s) = channel();
        let (tx_c, rx_c) = channel();
        let server = Server {
            socket: srv,
            connected: connected.clone(),
            client: None,
            signal: None,
            data: None,
            events: tx_e,
            recv: rx_s,
            send: tx_r,
            connect: rx_c,
            stop: rx_end,
        };
        std::thread::spawn(move || server.run());
        Self {
            connected,
            events: rx_e,
            connect: tx_c,
            signal: tx_end,
            recv: rx_r,
            send: tx_s,
            port
        }
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn event(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }

    pub fn connect(&self, ip: Ipv4Addr, port: u16) {
        log::info!("Connecting to peer {ip:?}:{port}");
        self.connect.send((ip, port)).ok();
    }

    pub fn disconnect(&self) {
        self.signal.send(Event::Disconnect).ok();
    }

    pub fn send(&self, data: u8) { self.send.send(data).ok(); }
    pub fn recv(&self) -> Option<u8> {
        self.recv.try_recv().ok()
    }
}
