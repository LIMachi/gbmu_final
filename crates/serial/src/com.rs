use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use std::sync::mpsc::{Sender, Receiver, channel};

pub enum Event { Stop, Connected(SocketAddr) }

struct Client {
    inner: TcpStream,
    data: Sender<u8>,
}

impl Client {
    pub fn spawn(client: TcpStream, data: Sender<u8>) -> Self {
        Self { inner: client, data }
    }
    pub fn run(mut self) {
        std::thread::spawn(move || {
            self.inner.set_nonblocking(false).expect("no block");
            let mut buf = [0; 1];
            loop {
                match self.inner.read_exact(&mut buf) {
                    Ok(()) => match self.data.send(buf[0]) {
                        Err(e) => { log::warn!("client: failed to send down recv data"); break },
                        _ => {}
                    },
                    Err(e) => {
                        match e.kind() {
                            ErrorKind::ConnectionReset => break,
                            ErrorKind::WouldBlock => {},
                            e => log::warn!("client: {e:?}")
                        }
                    }
                }
            }
            log::info!("disconnected");
        });
    }
}

pub struct Serial {
    stop: Sender<Event>,
    events: Receiver<Event>,
    connect: Sender<(Ipv4Addr, u16)>,
    recv: Receiver<u8>,
    send: Sender<u8>,
    connected: Arc<Mutex<bool>>
}

impl Serial {
    pub fn build() -> Self {
        let connected = Arc::new(Mutex::new(false));
        let mut srv = TcpListener::bind(
            &(27542..27552).map(|port| SocketAddr::from(([0, 0, 0, 0], port)))
                .collect::<Vec<SocketAddr>>()[..]).expect("failed to find available port");
        srv.set_nonblocking(true).expect("failed to set nonblocking");
        log::info!("server started on {:?}", srv.local_addr());
        let (tx_e, rx_e) = channel();
        let (tx_end, rx_end) = channel();
        let (tx_r, rx_r) = channel();
        let (tx_s, rx_s) = channel();
        let (tx_c, rx_c) = channel();
        let clone = connected.clone();
        std::thread::spawn(move || {
            let mut connected = clone;
            let mut rx = None;
            let mut client = None;
            loop {
                match srv.accept() {
                    Ok((stream, addr)) => {
                        let (tx, r) = channel();
                        rx = Some(r);
                        client = Some(stream.try_clone().expect("failed to clone socket"));
                        Client::spawn(stream, tx).run();
                        *connected.lock().unwrap() = true;
                        tx_e.send(Event::Connected(addr)).ok();
                    },
                    Err(e) if e.kind() != ErrorKind::WouldBlock => log::warn!("connection refused {e:?}"),
                    _ => {}
                };
                rx.as_mut().and_then(|x| x.try_recv().ok())
                    .and_then(|x| tx_r.send(x).ok());
                if let Some(client) = client.as_mut() {
                    if let Some(data) = rx_s.try_recv().ok() {
                        if let Err(e) = client.write(&[data]) {
                            log::warn!("error client send: {e:?}");
                        }
                    }
                }
                if let Some((addr, port)) = rx_c.try_recv().ok() {
                    TcpStream::connect((addr, port))
                        .map(|x| {
                            let (tx, r) = channel();
                            rx = Some(r);
                            client = Some(x.try_clone().expect("failed to clone socket"));
                            tx_e.send(Event::Connected(SocketAddr::new(IpAddr::V4(addr), port))).ok();
                            Client::spawn(x, tx).run();
                            *connected.lock().unwrap() = true;
                        }).ok();
                }
                if let Some(_) = rx_end.try_recv().ok() {
                    break;
                }
            }
        });
        Self {
            connected,
            events: rx_e,
            connect: tx_c,
            stop: tx_end,
            recv: rx_r,
            send: tx_s
        }
    }

    pub fn connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    pub fn event(&mut self) -> Option<Event> {
        self.events.try_recv().ok()
    }

    pub fn connect(&mut self, ip: Ipv4Addr, port: u16) {
        self.connect.send((ip, port)).ok();
    }

    pub fn send(&mut self, data: u8) { self.send.send(data).ok(); }
    pub fn recv(&mut self) -> Option<u8> { self.recv.try_recv().ok() }
}

impl Drop for Serial {
    fn drop(&mut self) { self.stop.send(Event::Stop).ok(); }
}
