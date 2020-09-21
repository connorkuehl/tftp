use std::convert::AsRef;
use std::io::{self, ErrorKind};
use std::mem::size_of;

pub trait FromBytes: Sized {
    type Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Self::Error>;
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

pub struct Bytes<T>(T);

impl Bytes<u16> {
    pub fn new(val: u16) -> Self {
        Self(val)
    }

    pub fn into_inner(self) -> u16 {
        self.0
    }
}

impl FromBytes for Bytes<u16> {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<Self> {
        let bytes = bytes.as_ref();

        if bytes.len() > size_of::<u16>() {
            return Err(ErrorKind::InvalidInput.into());
        }

        let mut bs = [0u8; size_of::<u16>()];
        bs.copy_from_slice(&bytes[..]);
        let be = u16::from_be_bytes(bs);

        Ok(Self(be))
    }
}

impl IntoBytes for Bytes<u16> {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = self.0.to_be_bytes();
        bytes.to_vec()
    }
}
