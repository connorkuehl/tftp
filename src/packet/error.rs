use std::convert::AsRef;
use std::fmt;
use std::io::{self, ErrorKind, Result};
use std::mem::size_of;

use crate::bytes::{Bytes, FromBytes, IntoBytes};
use crate::packet::sealed::Packet;
use crate::packet::opcode::Opcode;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Code {
    NotDefined = 0,
    FileNotFound = 1,
    AccessViolation = 2,
    DiskFull = 3,
    IllegalOperation = 4,
    UnknownTid = 5,
    FileAlreadyExists = 6,
    NoSuchUser = 7,
}

impl Code {
    pub fn from_u16(val: u16) -> Result<Self> {
        Ok(match val {
            v if v == 0 => Code::NotDefined,
            v if v == 1 => Code::FileNotFound,
            v if v == 2 => Code::AccessViolation,
            v if v == 3 => Code::DiskFull,
            v if v == 4 => Code::IllegalOperation,
            v if v == 5 => Code::UnknownTid,
            v if v == 6 => Code::FileAlreadyExists,
            v if v == 7 => Code::NoSuchUser,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl IntoBytes for Code {
    fn into_bytes(self) -> Vec<u8> {
        let val = self as u16;
        let bytes = Bytes::new(val);
        bytes.into_bytes()
    }
}

impl FromBytes for Code {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = Bytes::from_bytes(bytes)?;
        let code = Code::from_u16(bytes.into_inner())?;
        Ok(code)
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Code::NotDefined => "Not defined, see error message (if any)",
            Code::FileNotFound => "File not found",
            Code::AccessViolation => "Access violation",
            Code::DiskFull => "Disk full or allocation exceeded",
            Code::IllegalOperation => "Illegal TFTP operation",
            Code::UnknownTid => "Unknown transfer ID",
            Code::FileAlreadyExists => "File already exists",
            Code::NoSuchUser => "No such user",
        };

        write!(f, "{}", s)
    }
}

pub struct Error {
    code: Code,
    message: String,
}

impl Error {
    pub fn new<T: AsRef<str>>(code: Code, message: T) -> Self {
        Self {
            code,
            message: message.as_ref().to_string(),
        }
    }
}

impl Packet for Error {
    const OPCODE: Opcode = Opcode::Error;
}

impl FromBytes for Error {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        let (code, message) = bytes.split_at(size_of::<u16>());
        let code = Code::from_bytes(code)?;
        let message = Bytes::from_bytes(message)?;
        let message = message.into_inner();

        Ok(Self { code, message })
    }
}

impl IntoBytes for Error {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = self.code.into_bytes();
        let mut message = Bytes::new(self.message).into_bytes();
        bytes.append(&mut message);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_conversions() {
        assert_eq!(Code::from_u16(0).unwrap(), Code::NotDefined);
        assert_eq!(Code::from_u16(1).unwrap(), Code::FileNotFound);
        assert_eq!(Code::from_u16(2).unwrap(), Code::AccessViolation);
        assert_eq!(Code::from_u16(3).unwrap(), Code::DiskFull);
        assert_eq!(Code::from_u16(4).unwrap(), Code::IllegalOperation);
        assert_eq!(Code::from_u16(5).unwrap(), Code::UnknownTid);
        assert_eq!(Code::from_u16(6).unwrap(), Code::FileAlreadyExists);
        assert_eq!(Code::from_u16(7).unwrap(), Code::NoSuchUser);
        assert!(Code::from_u16(8).is_err());
    }

    #[test]
    fn test_from_bytes() {
        let input = &[0, 5, b'm', b'e', b's', b's', b'a', b'g', b'e', b'\0'];
        let actual = Error::from_bytes(&input[..]).unwrap();

        assert_eq!(actual.code, Code::UnknownTid);
        assert_eq!(actual.message.as_str(), "message");

        let input = &[0, 0, b'\0'];
        let actual = Error::from_bytes(&input[..]).unwrap();
        assert_eq!(actual.code, Code::NotDefined);
        assert_eq!(actual.message.as_str(), "");

        assert!(Error::from_bytes(&[0, 1]).is_err());
        assert!(Error::from_bytes(&[2, b'\0']).is_err());
    }

    #[test]
    fn test_into_bytes() {
        let error = Error {
            code: Code::AccessViolation,
            message: format!("{}", Code::AccessViolation),
        };

        let bytes = error.into_bytes();
        assert_eq!(&bytes[..], &[0, 2, b'A', b'c', b'c', b'e', b's', b's', b' ', b'v', b'i', b'o', b'l', b'a', b't', b'i', b'o', b'n', b'\0']);
    }
}
