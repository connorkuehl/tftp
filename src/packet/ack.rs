use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use super::Block;
use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::opcode::Opcode;
use crate::packet::sealed::Packet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ack {
    pub block: Block,
}

impl Ack {
    pub fn new(block: Block) -> Self {
        Self { block }
    }
}

impl Packet for Ack {
    const OPCODE: Opcode = Opcode::Ack;
}

impl FromBytes for Ack {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();

        let split_at = size_of::<Block>();

        if bytes.len() != split_at {
            return Err(ErrorKind::InvalidInput.into());
        }

        let block = &bytes[..split_at];
        let block = Block::from_bytes(block)?;

        Ok(Self { block })
    }
}

impl IntoBytes for Ack {
    fn into_bytes(self) -> Vec<u8> {
        self.block.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input = &[0, 1];
        let actual = Ack::from_bytes(&input[..]).unwrap();

        assert_eq!(actual.block.0, 1);
        assert!(Ack::from_bytes(&[1]).is_err());
        assert!(Ack::from_bytes(&[1, 2, 3]).is_err());
    }

    #[test]
    fn test_into_bytes() {
        let ack = Ack { block: Block(23) };

        let bytes = ack.into_bytes();
        assert_eq!(&bytes[..], &[0, 23]);
    }
}
