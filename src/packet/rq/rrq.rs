use std::io::{self, Result};

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::mode::Mode;
use crate::packet::opcode::Opcode;
use crate::packet::sealed::Packet;
use super::Rq;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rrq(Rq);

impl Rrq {
    pub fn new<T: AsRef<str>>(filename: T, mode: Mode) -> Self {
        let filename = filename.as_ref().to_string();
        Self(Rq { filename, mode })
    }
}

impl Packet for Rrq {
    const OPCODE: Opcode = Opcode::Rrq;
}

impl FromBytes for Rrq {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let rq = Rq::from_bytes(bytes)?;

        Ok(Self(rq))
    }
}

impl IntoBytes for Rrq {
    fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}
