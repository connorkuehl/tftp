use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use super::Block;
use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::opcode::Opcode;
use crate::packet::sealed::Packet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub block: Block,
    pub data: Vec<u8>,
}

impl Data {
    pub fn new<T: AsRef<[u8]>>(block: Block, data: T) -> Self {
        Self {
            block,
            data: data.as_ref().to_vec(),
        }
    }
}

impl Packet for Data {
    const OPCODE: Opcode = Opcode::Data;
}

impl FromBytes for Data {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();

        let split_at = size_of::<Block>();
        if split_at > bytes.len() {
            return Err(ErrorKind::InvalidInput.into());
        }

        let (block, data) = bytes.split_at(split_at);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input = vec![0x00, 0x01, b'p', b'o', b't', b'a', b't', b'o'];
        let actual = Data::from_bytes(&input[..]).unwrap();

        assert_eq!(actual.block, Block(1));
        assert_eq!(actual.data, b"potato");

        let input = &[0, 2];
        let actual = Data::from_bytes(&input[..]).unwrap();

        assert_eq!(actual.block, Block(2));
        assert_eq!(actual.data, &[]);

        assert!(Data::from_bytes(&[0]).is_err());
    }

    #[test]
    fn test_into_bytes() {
        let data = Data {
            block: Block(50),
            data: vec![1, 2, 3],
        };

        let bytes = data.into_bytes();
        assert_eq!(&bytes[..], &[0, 50, 1, 2, 3]);
    }
}
