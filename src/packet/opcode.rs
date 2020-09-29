//! Describes the opcodes defined by RFC 1350.

use std::convert::AsRef;
use std::fmt;
use std::io::{self, ErrorKind, Result};

use crate::bytes::{Bytes, FromBytes, IntoBytes};

/// An integer identifier for the type of TFTP packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    /// Read request.
    Rrq = 1,

    /// Write request.
    Wrq = 2,

    /// Data.
    Data = 3,

    /// Acknowledges successful receipt of a `Data` packet.
    Ack = 4,

    /// A courtesy packet to indicate the peer has experienced an error
    /// and will not complete the transmission.
    Error = 5,
}

impl Opcode {
    /// Tries to produce an `Opcode` from a `u16`.
    pub fn from_u16(val: u16) -> Result<Self> {
        Ok(match val {
            v if v == 1 => Opcode::Rrq,
            v if v == 2 => Opcode::Wrq,
            v if v == 3 => Opcode::Data,
            v if v == 4 => Opcode::Ack,
            v if v == 5 => Opcode::Error,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl IntoBytes for Opcode {
    fn into_bytes(self) -> Vec<u8> {
        let val = self as u16;
        let bytes = Bytes::new(val);
        bytes.into_bytes()
    }
}

impl FromBytes for Opcode {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = Bytes::from_bytes(bytes)?;
        let op = Opcode::from_u16(bytes.into_inner())?;
        Ok(op)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Opcode::Rrq => "RRQ",
            Opcode::Wrq => "WRQ",
            Opcode::Data => "DATA",
            Opcode::Ack => "ACK",
            Opcode::Error => "ERROR",
        };

        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversions() {
        assert!(Opcode::from_u16(0).is_err());
        assert_eq!(Opcode::from_u16(1).unwrap(), Opcode::Rrq);
        assert_eq!(Opcode::from_u16(2).unwrap(), Opcode::Wrq);
        assert_eq!(Opcode::from_u16(3).unwrap(), Opcode::Data);
        assert_eq!(Opcode::from_u16(4).unwrap(), Opcode::Ack);
        assert_eq!(Opcode::from_u16(5).unwrap(), Opcode::Error);
        assert!(Opcode::from_u16(6).is_err());

        assert_eq!(Opcode::Ack.into_bytes(), vec![0x00, 0x04]);
        assert_eq!(Opcode::from_bytes(&[0x00, 0x01]).unwrap(), Opcode::Rrq);
    }
}
