use std::io::{self, Read, Result, Write};
use std::mem::size_of;
use std::net::UdpSocket;

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::*;

/*
 * TODO: Probably add support for timeouts and retransmissions */

pub struct Connection {
    socket: UdpSocket,
}

impl Connection {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }

    pub fn get<W: Write>(&self, mut writer: W) -> Result<W> {
        let mut blocks_recvd = 1;

        loop {
            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = self.socket.recv(&mut buf)?;

            let data = match Packet::<Data>::from_bytes(&buf[..bytes_recvd]) {
                Err(_) => {
                    let error = Packet::<Error>::from_bytes(&buf[..])?;
                    /* FIXME */
                    return Err(io::Error::new(io::ErrorKind::Other, "got error packet"));
                },
                Ok(d) => d,
            };

            let _ = writer.write(&data.body.data[..])?;

            blocks_recvd += 1;
            let ack = Packet::ack(Block::new(blocks_recvd));
            let _ = self.socket.send(&mut ack.into_bytes()[..])?;

            if data.body.data.len() < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(writer)
    }

    pub fn put<R: Read>(&self, reader: R) -> Result<()> {
        unimplemented!()
    }
}
