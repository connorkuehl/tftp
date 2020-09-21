use std::convert::AsRef;
use std::fmt;
use std::io::{self, ErrorKind, Result};

use crate::bytes::{FromBytes, IntoBytes};

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

/// An `Error` packet is a courtesy packet that is sent prior to terminating
/// the TFTP connection due to an unrecoverable error.
#[derive(Debug)]
pub struct Error {
    /// An integer code that describes the error.
    pub code: Code,

    /// A human readable description of the error.
    pub message: String,
}
