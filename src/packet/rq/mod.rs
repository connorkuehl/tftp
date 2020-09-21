use std::io::{self, ErrorKind, Result};

use crate::bytes::{Bytes, FromBytes, IntoBytes, FirstNul};
use super::mode::Mode;

mod rrq;
mod wrq;

struct Rq {
    filename: String,
    mode: Mode,
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
