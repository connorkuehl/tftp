//! A client-side connection to a TFTP server. Implementors can use this
//! to build a more fully-featured client application.

use std::io::{self, Read, Result, Write};
use std::iter::Iterator;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use rand::Rng;

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::connection::MIN_PORT_NUMBER;
use crate::packet::*;
use crate::RetransmissionConfig;

/// The initial state for building a `Client`.
pub struct New(());

/// An intermediate state for building a `Client`.Builder
///
/// At this point, the `Builder` has all the information
/// it needs to construct a client.
pub struct ConnectTo {
    server: Vec<SocketAddr>,
}

/// Builds a `Client`.
pub struct Builder<T> {
    data: T,
    retransmission_config: RetransmissionConfig,
    socket: UdpSocket,
}

/// Represents a single connection with a TFTP server.
pub struct Client {
    server: Vec<SocketAddr>,
    socket: UdpSocket,
    retransmission_config: RetransmissionConfig,
}

impl Builder<New> {
    /// Generates a Transfer ID (a bind address & port) and opens a `UdpSocket`
    /// for this connection.
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(MIN_PORT_NUMBER, u16::MAX);
        let bind_to = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(bind_to)?;

        Ok(Builder {
            data: New(()),
            retransmission_config: RetransmissionConfig::default(),
            socket,
        })
    }

    /// Stores the Transfer ID (address + port) of the server to connect to.
    pub fn connect_to<A: ToSocketAddrs>(self, server: A) -> Result<Builder<ConnectTo>> {
        Ok(Builder {
            data: ConnectTo {
                server: server.to_socket_addrs()?.collect(),
            },
            socket: self.socket,
            retransmission_config: self.retransmission_config,
        })
    }
}

impl Builder<ConnectTo> {
    /// Constructs the client.
    pub fn build(self) -> Client {
        Client {
            server: self.data.server,
            socket: self.socket,
            retransmission_config: self.retransmission_config,
        }
    }

    /// Creates an instance with a different socket from the original instance.
    pub fn try_clone(&self) -> Result<Self> {
        let new_sock_builder = Builder::new()?;
        let data = ConnectTo {
            server: self.data.server.clone(),
        };
        Ok(Builder {
            data,
            retransmission_config: self.retransmission_config,
            socket: new_sock_builder.socket,
        })
    }
}

impl<T> Builder<T> {
    /// Set the future client's retransmission config
    pub fn with_retransmission_config(
        mut self,
        retransmission_config: RetransmissionConfig,
    ) -> Result<Self> {
        self.retransmission_config = retransmission_config;
        self.socket
            .set_read_timeout(retransmission_config.timeout().copied())?;
        Ok(self)
    }
}

impl Client {
    /// Retrieves a file from the remote server.
    pub fn get<S: AsRef<str>, W: Write>(self, file: S, mode: Mode, writer: W) -> Result<W> {
        let rrq = Packet::rrq(file, mode);
        let _ = self
            .socket
            .send_to(&rrq.into_bytes()[..], &self.server[..])?;

        let mut buf = [0; MAX_PACKET_SIZE];
        let (_, server) = self.socket.peek_from(&mut buf)?;
        self.socket.connect(server)?;

        let conn = Connection::new(
            self.socket,
            self.retransmission_config.max_retransmissions(),
        );
        conn.get(writer)
    }

    /// Stores a file on the remote server.
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

        let conn = Connection::new(
            self.socket,
            self.retransmission_config.max_retransmissions(),
        );
        conn.put(reader)
    }
}
