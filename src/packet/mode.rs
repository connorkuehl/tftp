use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, ErrorKind, Result};
use std::str::FromStr;

use crate::bytes::{FromBytes, IntoBytes};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Mail,
    NetAscii,
    Octet,
}

impl Mode {
    pub fn into_string(self) -> String {
        let s = match self {
            Mode::Mail => "mail",
            Mode::NetAscii => "netascii",
            Mode::Octet => "octet",
        };

        s.to_string()
    }
}

impl IntoBytes for Mode {
    fn into_bytes(self) -> Vec<u8> {
        let c = CString::new(self.into_string()).unwrap();
        c.into_bytes_with_nul()
    }
}

impl FromBytes for Mode {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        let cstr = CStr::from_bytes_with_nul(bytes)
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;
        let s = cstr.to_str()
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;

        Mode::from_str(s)
    }
}

impl FromStr for Mode {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.to_ascii_lowercase();

        Ok(match s.as_str() {
            "mail" => Mode::Mail,
            "netascii" => Mode::NetAscii,
            "octet" => Mode::Octet,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Mode::Mail => "mail",
            Mode::NetAscii => "netascii",
            Mode::Octet => "octet",
        };

        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_conversions() {
        assert_eq!(Mode::from_str("mail").unwrap(), Mode::Mail);
        assert_eq!(Mode::from_str("netascii").unwrap(), Mode::NetAscii);
        assert_eq!(Mode::from_str("octet").unwrap(), Mode::Octet);
        assert_eq!(Mode::from_bytes(b"mail\0").unwrap(), Mode::Mail);
        assert_eq!(Mode::from_bytes(b"netascii\0").unwrap(), Mode::NetAscii);
        assert_eq!(Mode::from_bytes(b"octet\0").unwrap(), Mode::Octet);
        assert_eq!(Mode::from_str("NeTasCiI").unwrap(), Mode::NetAscii);
    }
}
