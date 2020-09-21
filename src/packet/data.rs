use std::io::{self, Result};
use std::mem::size_of;

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::sealed::Packet;
use crate::packet::opcode::Opcode;
use super::Block;

pub struct Data {
    pub block: Block,
    pub data: Vec<u8>,
}

impl Packet for Data {
    const OPCODE: Opcode = Opcode::Data;
}

impl FromBytes for Data {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        let (block, data) = bytes.split_at(size_of::<Block>());
        let block = Block::from_bytes(block)?;
        let data = data.to_vec();

        Ok(Self { block, data })
    }
}

impl IntoBytes for Data {
    fn into_bytes(self) -> Vec<u8> {
        let block = self.block.into_bytes();
        let mut data = self.data;
        let mut bytes = block;
        bytes.append(&mut data);
        bytes
    }
}
