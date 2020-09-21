use std::convert::AsRef;
use std::io::{self, Result};

use crate::bytes::{Bytes, FromBytes, IntoBytes};

mod ack;
mod data;
mod error;
mod mode;
mod opcode;
mod rq;

mod sealed {
    use crate::bytes::{FromBytes, IntoBytes};
    use super::opcode::Opcode;

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
