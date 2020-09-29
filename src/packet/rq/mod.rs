//! A "general" request.
//!
//! This module is meant to be specialized by submodules and
//! therefore it is not meant to be used directly.

use std::io::{self, ErrorKind, Result};

use super::mode::Mode;
use crate::bytes::{Bytes, FirstNul, FromBytes, IntoBytes};

mod rrq;
mod wrq;

pub use rrq::Rrq;
pub use wrq::Wrq;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rq {
    pub filename: String,
    pub mode: Mode,
}

impl FromBytes for Rq {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();

        let first_nul = match bytes.first_nul_idx() {
            Some(idx) => idx,
            None => return Err(ErrorKind::InvalidInput.into()),
        };

        /* want to include the nul byte of the filename in its slice */
        let split_at = first_nul + 1;
        let (filename, mode) = bytes.split_at(split_at);
        let filename = Bytes::from_bytes(filename)?;
        let filename = filename.into_inner();
        let mode = Mode::from_bytes(mode)?;

        Ok(Self { filename, mode })
    }
}

impl IntoBytes for Rq {
    fn into_bytes(self) -> Vec<u8> {
        let filename = Bytes::new(self.filename).into_bytes();
        let mut mode = self.mode.into_bytes();

        let mut bytes = filename;
        bytes.append(&mut mode);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input = b"alice-in-wonderland.txt\0netascii\0";
        let actual = Rq::from_bytes(&input[..]).unwrap();

        assert_eq!(actual.filename.as_str(), "alice-in-wonderland.txt");
        assert_eq!(actual.mode, Mode::NetAscii);

        assert!(Rq::from_bytes(b"no-nul").is_err());
        assert!(Rq::from_bytes(b"only-filename-here\0").is_err());
        assert!(Rq::from_bytes(b"only-filename-here\0nonul").is_err());
    }

    #[test]
    fn test_into_bytes() {
        let rq = Rq {
            filename: "alice-in-wonderland.txt".to_string(),
            mode: Mode::Octet,
        };

        let bytes = rq.into_bytes();
        assert_eq!(&bytes[..], b"alice-in-wonderland.txt\0octet\0");
    }
}
