use std::convert::AsRef;
use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use crate::bytes::{Bytes, FromBytes, IntoBytes};
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
