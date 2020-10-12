//! A TFTP server. Implementors can use this to build a more richly-featured
//! server application.

use std::fs::OpenOptions;
use std::io::{self, Result};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};

use rand::Rng;
use std::collections::HashSet;

use std::sync::{Arc, Mutex};

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::connection::MIN_PORT_NUMBER;
use crate::packet::*;

// Active clients type alias
type ClientsPool = Arc<Mutex<HashSet<SocketAddr>>>;

/// A TFTP server.
pub struct Server {
    socket: UdpSocket,
    serve_dir: PathBuf,
    active_clients_pool: ClientsPool,
}

impl Server {
    /// Creates a server configured to serve files from a given directory on
    /// a given address.
    pub fn new<A: ToSocketAddrs, P: AsRef<Path>>(bind_to: A, serve_from: P) -> Result<Self> {
        let socket = UdpSocket::bind(bind_to)?;
        Ok(Self {
            socket,
            serve_dir: serve_from.as_ref().to_owned(),
            active_clients_pool: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// Creates a server configured to serve files from a given directory on
    /// a given ip_address and a random port.
    /// On success the chosen port and the new `Server` instance are returned.
    pub fn random_port<A: AsRef<str>, P: AsRef<Path>>(
        ip_addr: A,
        serve_from: P,
    ) -> Result<(u16, Self)> {
        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(MIN_PORT_NUMBER, u16::MAX);
        let bind_to = format!("{}:{}", ip_addr.as_ref(), port);

        Self::new(bind_to, serve_from).map(|server| (port, server))
    }

    /// Waits for requests and returns a `Handler` instance.
    ///
    /// It is intended that implementors will loop on this method and may
    /// optionally use the decoupled `Handler` instance at a time of their
    /// choosing to service the request.
    ///
    /// This is designed to be friendly to server implementations of all types.
    /// For example, a server application that employs the use of a thread pool
    /// can simply send the `Handler` off into the thread pool to be serviced.Ack
    /* TODO: Maybe return option instead? */
    pub fn serve(&self) -> Result<Handler> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (nbytes, src_addr) = self.socket.recv_from(&mut buf)?;

        // Try to create a callback. Fail if this client TID is already in use.
        if !self.active_clients_pool.lock().unwrap().insert(src_addr) {
            return Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "Client TID taken.",
            ));
        }

        let rrq = Packet::<Rrq>::from_bytes(&buf[..nbytes]);
        let wrq = Packet::<Wrq>::from_bytes(&buf[..nbytes]);

        let direction = if let Ok(rq) = rrq {
            Direction::Get(rq)
        } else if let Ok(wq) = wrq {
            Direction::Put(wq)
        } else {
            let error = Packet::error(
                Code::IllegalOperation,
                format!("{}", Code::IllegalOperation),
            );
            let _ = self.socket.send(&error.into_bytes()[..]);
            return Err(io::ErrorKind::InvalidInput.into());
        };

        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(1001, u16::MAX);
        let addr = self.socket.local_addr()?.ip().to_string();
        let bind_to = format!("{}:{}", addr, port);

        Handler::new(
            bind_to,
            src_addr,
            direction,
            self.serve_dir.clone(),
            self.active_clients_pool.clone(),
        )
    }
}

enum Direction {
    Get(Packet<Rrq>),
    Put(Packet<Wrq>),
}

/// Handles a request from a single TFTP client.
pub struct Handler {
    socket: UdpSocket,
    direction: Direction,
    serve_dir: PathBuf,
    client: SocketAddr,
    clients_pool: Option<ClientsPool>,
}

impl Handler {
    fn new<A: ToSocketAddrs>(
        bind: A,
        client: SocketAddr,
        direction: Direction,
        serve_dir: PathBuf,
        clients_pool: ClientsPool,
    ) -> Result<Handler> {
        let socket = UdpSocket::bind(bind)?;
        socket.connect(client)?;
        let clients_pool = Some(clients_pool);
        Ok(Handler {
            socket,
            direction,
            serve_dir,
            client,
            clients_pool,
        })
    }

    /// Completes the handshake with the client and services the request.
    pub fn handle(mut self) -> Result<()> {
        let client = self.client.clone();
        let clients_pool = self.clients_pool.take().unwrap();
        let result = match self.direction {
            Direction::Get(_) => self.get(),
            Direction::Put(_) => self.put(),
        };
        clients_pool.lock().unwrap().remove(&client);
        result
    }

    fn get(self) -> Result<()> {
        if let Direction::Get(rrq) = self.direction {
            let f = match OpenOptions::new()
                .read(true)
                .open(self.serve_dir.join(rrq.body.0.filename))
            {
                Ok(f) => f,
                Err(e) => {
                    let error: Packet<Error> = e.into();
                    let _ = self.socket.send(&error.clone().into_bytes()[..]);
                    return Err(io::Error::from(error));
                }
            };
            let conn = Connection::new(self.socket);
            conn.put(f)?;
            Ok(())
        } else {
            panic!("handler direction is wrong");
        }
    }

    fn put(self) -> Result<()> {
        if let Direction::Put(wrq) = self.direction {
            let f = match OpenOptions::new()
                .write(true)
                .create_new(true)
                /* FIXME: Not sure why this hangs if create is not specified */
                .open(self.serve_dir.join(wrq.body.0.filename))
            {
                Ok(f) => f,
                Err(e) => {
                    let error: Packet<Error> = e.into();
                    let _ = self.socket.send(&error.clone().into_bytes()[..]);
                    return Err(io::Error::from(error));
                }
            };
            let ack = Packet::ack(Block::new(0));
            let _ = self.socket.send(&ack.into_bytes()[..])?;

            let conn = Connection::new(self.socket);
            conn.get(f)?;
            Ok(())
        } else {
            panic!("handler direction is wrong");
        }
    }
}

// These tests use hand-rolled partial client implmentations mostly copied from the proper implementation at client.rs.
// This is because we need to simulate incorrect client behaviors, and the public client api won't let us do that.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_use() {
        let exemplar = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/artifacts/alice-in-wonderland.txt"
        ));

        let server_addr = "127.0.0.1:62186";
        let wd = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/");
        let server = Server::new(server_addr, wd).unwrap();

        let server_thread = std::thread::spawn(move || {
            let h = server.serve().unwrap();
            h.handle().unwrap();
        });

        let bind_to = format!("0.0.0.0:62187");
        let socket = UdpSocket::bind(bind_to).unwrap();
        let rrq = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);
        socket
            .send_to(&rrq.clone().into_bytes(), server_addr)
            .unwrap();

        let mut buf = [0; MAX_PACKET_SIZE];
        let (_, server) = socket.peek_from(&mut buf).unwrap();
        socket.connect(server).unwrap();

        let conn = Connection::new(socket);

        let res: Vec<u8> = Vec::with_capacity(exemplar.len());
        let res = conn.get(res).unwrap();

        assert_eq!(&exemplar[..], &res[..]);

        server_thread.join().unwrap();
    }
    #[test]
    fn test_reuse_socket() {
        let exemplar = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/artifacts/alice-in-wonderland.txt"
        ));

        let server_addr = "127.0.0.1:62188";
        let wd = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/");
        let server = Server::new(server_addr, wd).unwrap();
        let n = 3;
        let server_thread = std::thread::spawn(move || {
            for _ in 0..n {
                let h = server.serve().unwrap();
                h.handle().unwrap();
            }
        });
        for _ in 0..n {
            let bind_to = format!("0.0.0.0:62189");
            let socket = UdpSocket::bind(bind_to).unwrap();

            let rrq = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);
            socket
                .send_to(&rrq.clone().into_bytes(), server_addr)
                .unwrap();

            let mut buf = [0; MAX_PACKET_SIZE];
            let (_, server) = socket.peek_from(&mut buf).unwrap();
            socket.connect(server).unwrap();

            let conn = Connection::new(socket);

            let res: Vec<u8> = Vec::with_capacity(exemplar.len());
            let res = conn.get(res).unwrap();

            assert_eq!(&exemplar[..], &res[..]);
        }

        server_thread.join().unwrap();
    }
    #[test]
    fn test_prevent_duplicate() {
        let exemplar = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/artifacts/alice-in-wonderland.txt"
        ));

        let server_addr = "127.0.0.1:62190";
        let wd = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/");
        let server = Server::new(server_addr, wd).unwrap();

        let server_thread = std::thread::spawn(move || {
            let h = server.serve().unwrap();
            let t1_handle = std::thread::spawn(move || {
                h.handle().unwrap();
            });
            match server.serve() {
                Err(e) => assert_eq!(e.kind(), io::ErrorKind::AddrNotAvailable),
                _ => panic!("Should get error."),
            };
            t1_handle.join().unwrap();
        });

        let bind_to = format!("0.0.0.0:62191");
        let socket = UdpSocket::bind(bind_to).unwrap();
        let rrq = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);

        socket
            .send_to(&rrq.clone().into_bytes(), server_addr)
            .unwrap();
        socket
            .send_to(&rrq.clone().into_bytes(), server_addr)
            .unwrap();

        let mut buf = [0; MAX_PACKET_SIZE];
        let (_, server) = socket.peek_from(&mut buf).unwrap();
        socket.connect(server).unwrap();

        let conn = Connection::new(socket);

        let res: Vec<u8> = Vec::with_capacity(exemplar.len());
        let res = conn.get(res).unwrap();

        assert_eq!(&exemplar[..], &res[..]);

        server_thread.join().unwrap();
    }
    #[test]
    fn test_concurrent_connections() {
        let exemplar = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/artifacts/alice-in-wonderland.txt"
        ));
        let n = 4;
        let server_addr = "127.0.0.1:62192";
        let wd = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts/");
        let server = Server::new(server_addr, wd).unwrap();

        let server_thread = std::thread::spawn(move || {
            let mut joins = Vec::with_capacity(n);
            for _ in 0..n {
                let h = server.serve().unwrap();
                joins.push(std::thread::spawn(move || {
                    h.handle().unwrap();
                }));
            }
            for join in joins {
                join.join().unwrap();
            }
        });
        let mut joins = Vec::with_capacity(n);
        for i in 0..n {
            joins.push(std::thread::spawn(move || {
                let bind_to = format!("0.0.0.0:{}", 62193 + i);
                let socket = UdpSocket::bind(bind_to).unwrap();

                let rrq = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);
                socket
                    .send_to(&rrq.clone().into_bytes(), server_addr)
                    .unwrap();

                let mut buf = [0; MAX_PACKET_SIZE];
                let (_, server) = socket.peek_from(&mut buf).unwrap();
                socket.connect(server).unwrap();

                let conn = Connection::new(socket);

                let res: Vec<u8> = Vec::with_capacity(exemplar.len());
                let res = conn.get(res).unwrap();

                assert_eq!(&exemplar[..], &res[..]);
            }));
        }
        for join in joins {
            join.join().unwrap();
        }

        server_thread.join().unwrap();
    }
}
