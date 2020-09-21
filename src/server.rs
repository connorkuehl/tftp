use std::env;
use std::io::{self, Result};
use std::fs::OpenOptions;
use std::net::{ToSocketAddrs, UdpSocket};
use std::path::Path;

use rand::Rng;

use crate::bytes::{FromBytes, IntoBytes};
use crate::connection::Connection;
use crate::packet::*;

pub struct Server(UdpSocket);

impl Server {
    pub fn new<A: ToSocketAddrs, P: AsRef<Path>>(bind_to: A, serve_from: P) -> Result<Self> {
        let socket = UdpSocket::bind(bind_to)?;
        env::set_current_dir(serve_from)?;

        Ok(Self(socket))
    }

    pub fn serve(&self) -> Result<Handler> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (nbytes, src_addr) = self.0.recv_from(&mut buf)?;
        let rrq = Packet::<Rrq>::from_bytes(&buf[..nbytes]);
        let wrq = Packet::<Wrq>::from_bytes(&buf[..nbytes]);

        let direction = if let Ok(rq) = rrq {
            Some(Direction::Get(rq))
        } else if let Ok(wq) = wrq {
            Some(Direction::Put(wq))
        } else {
            None
        };

        let direction = match direction {
            None => return Err(io::ErrorKind::InvalidInput.into()),
            Some(d) => d,
        };

        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(1001, u16::MAX);
        let bind_to = format!("127.0.0.1:{}", port);

        Handler::new(bind_to, src_addr, direction)
    }
}

enum Direction {
    Get(Packet<Rrq>),
    Put(Packet<Wrq>),
}

pub struct Handler {
    socket: UdpSocket,
    direction: Direction,
}

impl Handler {
    fn new<A: ToSocketAddrs, B: ToSocketAddrs>(bind: A, client: B, direction: Direction) -> Result<Handler> {
        let socket = UdpSocket::bind(bind)?;
        socket.connect(client)?;

        Ok(Handler { socket, direction })
    }

    pub fn handle(self) -> Result<()> {
        match self.direction {
            Direction::Get(_) => self.get(),
            Direction::Put(_) => self.put(),
        }
    }

    fn get(self) -> Result<()> {
        if let Direction::Get(rrq) = self.direction {
            let f = OpenOptions::new()
                .read(true)
                .open(rrq.body.0.filename)?;
            let conn = Connection::new(self.socket);
            conn.put(f)?;
            Ok(())
        } else {
            panic!("handler direction is wrong");
        }
    }

    fn put(self) -> Result<()> {
        if let Direction::Put(wrq) = self.direction {
            let f = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(wrq.body.0.filename)?;
            let ack = Packet::ack(Block::new(0));
            let _ = self.socket.send(&mut ack.into_bytes()[..])?;

            let conn = Connection::new(self.socket);
            conn.get(f)?;
            Ok(())
        } else {
            panic!("handler direction is wrong");
        }
    }
}
