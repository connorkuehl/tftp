//!

use std::convert::TryFrom;
use std::io::{Read, Result, Write};
use std::net::UdpSocket;

use crate::packet::*;

pub trait Direction {}

pub struct Get<T: Write>(T);

impl<T: Write> Direction for Get<T> {}

pub struct Put<T: Read>(T);

impl<T: Read> Direction for Put<T> {}

pub struct Connection<T: Direction> {
    socket: UdpSocket,
    direction: T,
}

impl<T: Write> Connection<Get<T>> {
    pub fn new(with: UdpSocket, writer: T) -> Connection<Get<T>> {
        Connection {
            socket: with,
            direction: Get(writer),
        }
    }

    pub fn get(mut self) -> Result<T> {
        /* FIXME: this doesn't even bother with timeouts. */

        let mut last_block = 0;
        loop {
            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_read = self.socket.recv(&mut buf)?;
            assert!(bytes_read >= 2);

            /* FIXME: 2 issues here:
             * 1) taking ownership of the bytes is awkward in the event that
             * parsing the packet fails
             *
             * 2) this should be improved to make it easier to read an error
             * packet too. */
            let data: Packet<Data> = Packet::try_from(buf.to_vec())?;
            self.direction.0.write_all(&data.body.data[..])?;
            last_block += 1;

            let ack = Packet::<Ack>::new(last_block);
            let buf: Vec<u8> = ack.into();
            self.socket.send(&buf[..])?;

            if data.body.data.len() < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(self.direction.0)
    }
}

impl<T: Read> Connection<Put<T>> {
    pub fn new(with: UdpSocket, reader: T) -> Connection<Put<T>> {
        Connection {
            socket: with,
            direction: Put(reader),
        }
    }

    pub fn put(mut self) -> Result<()> {
        /*
         * FIXME: TODO: retransmit timed out packets
         * FIXME: ensure the ack matches the last packet sent */
        loop {
            let mut buf = [0; MAX_PAYLOAD_SIZE];
            let bytes_read = self.direction.0.read(&mut buf[..])?;

            let data: Packet<Data> = Packet::try_from(buf.to_vec())?;
            let bytes: Vec<u8> = data.into();
            let _ = self.socket.send(&bytes[..])?;

            let mut buf = [0; MAX_PACKET_SIZE];
            let _ = self.socket.recv(&mut buf[..])?;
            let _: Packet<Ack> = Packet::try_from(buf.to_vec())?;

            if bytes_read < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(())
    }
}
