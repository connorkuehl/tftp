//! A TFTP server. Implementors can use this to build a more richly-featured
//! server application.

use std::env;
use std::fs::OpenOptions;
use std::io::{self, Result};
use std::net::{ToSocketAddrs, UdpSocket};
use std::path::Path;

use rand::Rng;

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::packet::*;

/// A TFTP server.
pub struct Server(UdpSocket);

impl Server {
    /// Creates a server configured to serve files from a given directory on
    /// a given address.
    pub fn new<A: ToSocketAddrs, P: AsRef<Path>>(bind_to: A, serve_from: P) -> Result<Self> {
        let socket = UdpSocket::bind(bind_to)?;
        env::set_current_dir(serve_from)?;

        Ok(Self(socket))
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
        let (nbytes, src_addr) = self.0.recv_from(&mut buf)?;
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
            let _ = self.0.send(&error.into_bytes()[..]);
            return Err(io::ErrorKind::InvalidInput.into());
        };

        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(1001, u16::MAX);
        let addr = self.0.local_addr()?.ip().to_string();
        let bind_to = format!("{}:{}", addr, port);

        Handler::new(bind_to, src_addr, direction)
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
}

impl Handler {
    fn new<A: ToSocketAddrs, B: ToSocketAddrs>(
        bind: A,
        client: B,
        direction: Direction,
    ) -> Result<Handler> {
        let socket = UdpSocket::bind(bind)?;
        socket.connect(client)?;

        Ok(Handler { socket, direction })
    }

    /// Completes the handshake with the client and services the request.
    pub fn handle(self) -> Result<()> {
        match self.direction {
            Direction::Get(_) => self.get(),
            Direction::Put(_) => self.put(),
        }
    }

    fn get(self) -> Result<()> {
        if let Direction::Get(rrq) = self.direction {
            let f = match OpenOptions::new().read(true).open(rrq.body.0.filename) {
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
                .create(true)
                /* FIXME: Not sure why this hangs if create is not specified */
                .truncate(true)
                .open(wrq.body.0.filename)
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
