use std::convert::AsRef;
use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use crate::bytes::{Bytes, FromBytes, IntoBytes};
use ack::Ack;
use data::Data;
use error::{Code, Error};
use rq::{Rrq, Wrq};
use mode::Mode;
use opcode::Opcode;

mod ack;
mod data;
mod error;
mod mode;
mod opcode;
mod rq;

mod sealed {
    use crate::bytes::{FromBytes, IntoBytes};
    use crate::packet::opcode::Opcode;

    pub trait Packet: FromBytes + IntoBytes {
        const OPCODE: Opcode;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Block(u16);

impl FromBytes for Block {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        let block = Bytes::from_bytes(bytes)?;

        Ok(Self(block.into_inner()))
    }
}

impl IntoBytes for Block {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = Bytes::new(self.0);
        bytes.into_bytes()
    }
}

pub struct Packet<T: sealed::Packet> {
    header: Opcode,
    body: T,
}

impl<T: sealed::Packet> Packet<T> {
    fn new(body: T) -> Self {
        Self {
            header: T::OPCODE,
            body,
        }
    }
}

impl Packet<Rrq> {
    pub fn rrq(filename: String, mode: Mode) -> Self {
        let rrq = Rrq::new(filename, mode);

        Self::new(rrq)
    }
}

impl Packet<Wrq> {
    pub fn wrq(filename: String, mode: Mode) -> Self {
        let wrq = Wrq::new(filename, mode);

        Self::new(wrq)
    }
}

impl Packet<Data> {
    pub fn data<T: AsRef<[u8]>>(block: Block, data: T) -> Self {
        let data = Data::new(block, data);

        Self::new(data)
    }
}

impl Packet<Ack> {
    pub fn ack(block: Block) -> Self {
        let ack = Ack::new(block);

        Self::new(ack)
    }
}

impl Packet<Error> {
    pub fn error<T: AsRef<str>>(code: Code, message: T) -> Self {
        let error = Error::new(code, message);

        Self::new(error)
    }
}

impl<T: sealed::Packet> FromBytes for Packet<T> {
    type Error = io::Error;

    fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self> {
        let bytes = bytes.as_ref();
        let (header, body) = bytes.split_at(size_of::<u16>());
        let opcode = Opcode::from_bytes(header)?;
        if opcode != T::OPCODE {
            return Err(ErrorKind::InvalidData.into());
        }

        /* FIXME: Remove map_err and just use `?` */
        let body = T::from_bytes(body)
            .map_err(|_| -> io::Error { ErrorKind::InvalidData.into() })?;

        Ok(Self { header: opcode, body })
    }
}

impl<T: sealed::Packet> IntoBytes for Packet<T> {
    fn into_bytes(self) -> Vec<u8> {
        let mut body = self.body.into_bytes();
        let mut bytes = self.header.into_bytes();
        bytes.append(&mut body);
        bytes
    }
}
