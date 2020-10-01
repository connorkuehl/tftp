//! Parsing and creating TFTP packets.

use std::convert::AsRef;
use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use crate::bytes::{Bytes, FromBytes, IntoBytes};
pub use ack::Ack;
pub use data::Data;
pub use error::{Code, Error};
pub use mode::Mode;
pub use opcode::Opcode;
pub use rq::{Rrq, Wrq};

mod ack;
mod data;
mod error;
mod mode;
mod opcode;
mod rq;

/// The maximum number of bytes carried in a `Data` packet.
pub const MAX_PAYLOAD_SIZE: usize = 512;

/// The total size of a TFTP packet. (512 + 2 byte opcode + 2 byte block ID)
pub const MAX_PACKET_SIZE: usize = 516;

mod sealed {
    use crate::bytes::{FromBytes, IntoBytes};
    use crate::packet::opcode::Opcode;

    pub trait Packet: FromBytes + IntoBytes {
        const OPCODE: Opcode;
    }
}

/// An identifier for a `Data` block and its corresponding `Ack` packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Block(u16);

impl Block {
    /// Creates a new `Block`.
    pub fn new(val: u16) -> Self {
        Self(val)
    }
}

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

/// A TFTP packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packet<T: sealed::Packet> {
    /// Describes the packet.
    header: Opcode,

    /// Contains the packet payload.
    body: T,
}

impl<T: sealed::Packet> Packet<T> {
    fn new(body: T) -> Self {
        Self {
            header: T::OPCODE,
            body,
        }
    }

    /// Returns the opcode for the packet
    pub fn header(&self) -> Opcode {
        self.header
    }

    /// Returns a reference to the packet's payload
    pub fn body(&self) -> &T {
        &self.body
    }
}

impl Packet<Rrq> {
    /// Creates a new read request packet.
    pub fn rrq<T: AsRef<str>>(filename: T, mode: Mode) -> Self {
        let rrq = Rrq::new(filename, mode);

        Self::new(rrq)
    }
}

impl Packet<Wrq> {
    /// Creates a new write request packet.
    pub fn wrq<T: AsRef<str>>(filename: T, mode: Mode) -> Self {
        let wrq = Wrq::new(filename, mode);

        Self::new(wrq)
    }
}

impl Packet<Data> {
    /// Creates a new data packet.
    pub fn data<T: AsRef<[u8]>>(block: Block, data: T) -> Self {
        let data = Data::new(block, data);

        Self::new(data)
    }
}

impl Packet<Ack> {
    /// Creates a new ack packet.
    pub fn ack(block: Block) -> Self {
        let ack = Ack::new(block);

        Self::new(ack)
    }
}

impl Packet<Error> {
    /// Creates a new error packet.
    pub fn error<T: AsRef<str>>(code: Code, message: T) -> Self {
        let error = Error::new(code, message);

        Self::new(error)
    }
}

impl From<io::Error> for Packet<Error> {
    fn from(err: io::Error) -> Packet<Error> {
        let code = match err.kind() {
            ErrorKind::NotFound => Code::FileNotFound,
            ErrorKind::PermissionDenied => Code::AccessViolation,
            ErrorKind::AlreadyExists => Code::FileAlreadyExists,
            _ => Code::NotDefined,
        };

        let message = format!("{}", code);

        Packet::error(code, message)
    }
}

impl From<Packet<Error>> for io::Error {
    fn from(err: Packet<Error>) -> io::Error {
        let kind = match err.body.code() {
            Code::FileNotFound => ErrorKind::NotFound,
            Code::AccessViolation => ErrorKind::PermissionDenied,
            Code::FileAlreadyExists => ErrorKind::AlreadyExists,
            _ => ErrorKind::Other,
        };

        io::Error::new(kind, err.body.message())
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
        let body =
            T::from_bytes(body).map_err(|_| -> io::Error { ErrorKind::InvalidData.into() })?;

        Ok(Self {
            header: opcode,
            body,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrq() {
        let rrq = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);
        assert_eq!(rrq.header, Opcode::Rrq);

        let op = vec![0, 1];
        let mut mode = b"netascii\0".to_vec();
        let mut filename = b"alice-in-wonderland.txt\0".to_vec();
        let mut bytes = op;
        bytes.append(&mut filename);
        bytes.append(&mut mode);
        assert_eq!(bytes, rrq.into_bytes());

        let expected = Packet::rrq("alice-in-wonderland.txt", Mode::NetAscii);
        let actual = Packet::<Rrq>::from_bytes(&bytes[..]).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_wrq() {
        let wrq = Packet::wrq("alice-in-wonderland.txt", Mode::Mail);
        assert_eq!(wrq.header, Opcode::Wrq);

        let op = vec![0, 2];
        let mut mode = b"mail\0".to_vec();
        let mut filename = b"alice-in-wonderland.txt\0".to_vec();
        let mut bytes = op;
        bytes.append(&mut filename);
        bytes.append(&mut mode);
        assert_eq!(bytes, wrq.into_bytes());

        let expected = Packet::wrq("alice-in-wonderland.txt", Mode::Mail);
        let actual = Packet::<Wrq>::from_bytes(&bytes[..]).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_data() {
        let data = Packet::data(Block(25), &[1, 2, 3]);
        assert_eq!(data.header, Opcode::Data);

        let op = vec![0, 3];
        let mut block = vec![0, 25];
        let mut dat = vec![1, 2, 3];
        let mut bytes = op;
        bytes.append(&mut block);
        bytes.append(&mut dat);
        assert_eq!(bytes, data.into_bytes());

        let expected = Packet::data(Block(25), &[1, 2, 3]);
        let actual = Packet::<Data>::from_bytes(&bytes[..]).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ack() {
        let ack = Packet::ack(Block(1));
        assert_eq!(ack.header, Opcode::Ack);

        let op = vec![0, 4];
        let mut block = vec![0, 1];
        let mut bytes = op;
        bytes.append(&mut block);
        assert_eq!(bytes, ack.into_bytes());

        let expected = Packet::ack(Block(1));
        let actual = Packet::<Ack>::from_bytes(&bytes[..]).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_error() {
        let error = Packet::error(Code::FileNotFound, "file not found");
        assert_eq!(error.header, Opcode::Error);

        let op = vec![0, 5];
        let mut code = vec![0, 1];
        let mut message = b"file not found\0".to_vec();
        let mut bytes = op;
        bytes.append(&mut code);
        bytes.append(&mut message);
        assert_eq!(bytes, error.into_bytes());

        let expected = Packet::error(Code::FileNotFound, "file not found");
        let actual = Packet::<Error>::from_bytes(&bytes[..]).unwrap();
        assert_eq!(expected, actual);
    }
}
