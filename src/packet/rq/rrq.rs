//! A Read Request indicates that a peer wants to receive a file.

use std::io::{self, Result};

use super::Rq;
use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::mode::Mode;
use crate::packet::opcode::Opcode;
use crate::packet::sealed::Packet;

/// A read request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rrq(Rq);

impl Rrq {
    /// Creates a new `Rrq`.
    pub fn new<T: AsRef<str>>(filename: T, mode: Mode) -> Self {
        let filename = filename.as_ref().to_string();
        Self(Rq { filename, mode })
    }

    /// Returns a reference to the inner request
    pub fn request(&self) -> &Rq {
        &self.0
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
