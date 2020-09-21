use std::convert::AsRef;
use std::ffi::{CStr, CString};
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

impl<T> Bytes<T> {
    pub fn new(val: T) -> Self {
        Self(val)
    }

    pub fn into_inner(self) -> T {
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

impl FromBytes for Bytes<String> {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> io::Result<Self> {
        let bytes = bytes.as_ref();
        let cstr = CStr::from_bytes_with_nul(bytes)
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;
        let s = cstr
            .to_str()
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;

        Ok(Self(s.to_string()))
    }
}

impl IntoBytes for Bytes<String> {
    fn into_bytes(self) -> Vec<u8> {
        let c = CString::new(self.0).unwrap();
        c.into_bytes_with_nul()
    }
}

pub trait FirstNul {
    fn first_nul_idx(&self) -> Option<usize>;
}

impl<T: AsRef<[u8]>> FirstNul for T {
    fn first_nul_idx(&self) -> Option<usize> {
        let bytes = self.as_ref();

        for (idx, byte) in bytes.iter().enumerate() {
            if *byte == b'\0' {
                return Some(idx);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_nul_idx() {
        let input = b"hello\0world\0";
        assert_eq!(Some(5), input.first_nul_idx());

        let input = b"no nul byte here!";
        assert_eq!(None, input.first_nul_idx());
    }

    #[test]
    fn test_from_bytes_u16() {
        let n = 55u16;
        let actual = Bytes::from_bytes(&n.to_be_bytes()[..]).unwrap();
        assert_eq!(n, actual.into_inner());
    }

    #[test]
    fn test_into_bytes_u16() {
        let n = 55u16;
        let b = Bytes::new(n);
        assert_eq!(&n.to_be_bytes()[..], &b.into_bytes()[..]);
    }

    #[test]
    fn test_from_bytes_string() {
        let b: Bytes<String> = Bytes::from_bytes(b"hello, world!\0").unwrap();
        let actual = b.into_inner();
        assert_eq!("hello, world!", actual.as_str());
    }

    #[test]
    fn test_into_bytes_string() {
        let b = Bytes::new("hello, world!".to_string());
        let actual = b.into_bytes();
        assert_eq!(b"hello, world!\0", &actual[..]);
    }
}
