use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::sealed::Packet;
use crate::packet::opcode::Opcode;
use super::Block;

pub struct Ack {
    pub block: Block,
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
