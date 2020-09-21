use std::io::{Read, Result, Write};
use std::net::{ToSocketAddrs, UdpSocket};

use rand::Rng;

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::packet::*;

pub struct New(UdpSocket);

pub struct ConnectTo<A: ToSocketAddrs> {
    server: A,
    socket: UdpSocket,
}

pub struct Client<T> {
    connection: T,
}

impl Client<New> {
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(1001, u16::MAX);
        let bind_to = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(bind_to)?;

        let client = Client {
            connection: New(socket),
        };

        Ok(client)
    }

    pub fn connect_to<A: ToSocketAddrs>(self, server: A) -> Result<Client<ConnectTo<A>>> {
        let with_server = ConnectTo {
            socket: self.connection.0,
            server,
        };

        Ok(Client {
            connection: with_server,
        })
    }
}

impl<A: ToSocketAddrs> Client<ConnectTo<A>> {
    pub fn get<S: AsRef<str>, W: Write>(self, file: S, mode: Mode, writer: W) -> Result<W> {
        let rrq = Packet::rrq(file, mode);
        let _ = self
            .connection
            .socket
            .send_to(&rrq.into_bytes()[..], self.connection.server)?;

        let mut buf = [0; MAX_PACKET_SIZE];
        let (_, server) = self.connection.socket.peek_from(&mut buf)?;
        self.connection.socket.connect(server)?;

        let conn = Connection::new(self.connection.socket);
        conn.get(writer)
    }

    pub fn put<S: AsRef<str>, R: Read>(self, file: S, mode: Mode, reader: R) -> Result<()> {
        let wrq = Packet::wrq(file, mode);
        let _ = self
            .connection
            .socket
            .send_to(&wrq.into_bytes()[..], self.connection.server)?;

        let mut buf = [0; MAX_PACKET_SIZE];
        let (nbytes, server) = self.connection.socket.recv_from(&mut buf)?;
        self.connection.socket.connect(server)?;

        let _ = Packet::<Ack>::from_bytes(&buf[..nbytes])?;

        let conn = Connection::new(self.connection.socket);
        conn.put(reader)
    }
}
