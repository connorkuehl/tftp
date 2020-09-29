use std::io::{self, Read, Result, Write};
use std::iter::Iterator;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use rand::Rng;

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::packet::*;

pub struct New {
    socket: UdpSocket,
}

pub struct ConnectTo {
    server: Vec<SocketAddr>,
    socket: UdpSocket,
}

pub struct Builder<T> {
    data: T,
}

pub struct Client {
    server: Vec<SocketAddr>,
    socket: UdpSocket,
}

impl Builder<New> {
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(1001, u16::MAX);
        let bind_to = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(bind_to)?;

        let data = New { socket };

        Ok(Builder { data })
    }

    pub fn connect_to<A: ToSocketAddrs>(self, server: A) -> Result<Builder<ConnectTo>> {
        let resolved = server.to_socket_addrs()?.collect();
        let data = ConnectTo {
            server: resolved,
            socket: self.data.socket,
        };

        Ok(Builder { data })
    }
}

impl Builder<ConnectTo> {
    pub fn build(self) -> Client {
        Client {
            server: self.data.server,
            socket: self.data.socket,
        }
    }
}

impl Client {
    pub fn get<S: AsRef<str>, W: Write>(self, file: S, mode: Mode, writer: W) -> Result<W> {
        let rrq = Packet::rrq(file, mode);
        let _ = self
            .socket
            .send_to(&rrq.into_bytes()[..], &self.server[..])?;

        let mut buf = [0; MAX_PACKET_SIZE];
        let (_, server) = self.socket.peek_from(&mut buf)?;
        self.socket.connect(server)?;

        let conn = Connection::new(self.socket);
        conn.get(writer)
    }

    pub fn put<S: AsRef<str>, R: Read>(self, file: S, mode: Mode, reader: R) -> Result<()> {
        let wrq = Packet::wrq(file, mode);
        let _ = self
            .socket
            .send_to(&wrq.into_bytes()[..], &self.server[..])?;

        let mut buf = [0; MAX_PACKET_SIZE];
        let (nbytes, server) = self.socket.recv_from(&mut buf)?;
        self.socket.connect(server)?;

        let _ = match Packet::<Ack>::from_bytes(&buf[..nbytes]) {
            Ok(a) => a,
            Err(e) => {
                let error: Packet<Error> = e.into();
                return Err(io::Error::from(error));
            }
        };

        let conn = Connection::new(self.socket);
        conn.put(reader)
    }
}
