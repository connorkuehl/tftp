use std::convert::TryFrom;
use std::io::{self, ErrorKind, Result};

use super::*;

impl TryFrom<u16> for Opcode {
    type Error = io::Error;

    fn try_from(val: u16) -> Result<Opcode> {
        Ok(match val {
            o if o == Opcode::Rrq as u16 => Opcode::Rrq,
            o if o == Opcode::Wrq as u16 => Opcode::Wrq,
            o if o == Opcode::Data as u16 => Opcode::Data,
            o if o == Opcode::Ack as u16 => Opcode::Ack,
            o if o == Opcode::Error as u16 => Opcode::Error,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl From<Opcode> for u16 {
    fn from(op: Opcode) -> u16 {
        op as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversions() {
        assert_eq!(u16::from(Opcode::Rrq), 1);
        assert_eq!(u16::from(Opcode::Wrq), 2);
        assert_eq!(u16::from(Opcode::Data), 3);
        assert_eq!(u16::from(Opcode::Ack), 4);
        assert_eq!(u16::from(Opcode::Error), 5);

        assert!(Opcode::try_from(0).is_err());
        assert_eq!(Opcode::Rrq, Opcode::try_from(1).unwrap());
        assert_eq!(Opcode::Wrq, Opcode::try_from(2).unwrap());
        assert_eq!(Opcode::Data, Opcode::try_from(3).unwrap());
        assert_eq!(Opcode::Ack, Opcode::try_from(4).unwrap());
        assert_eq!(Opcode::Error, Opcode::try_from(5).unwrap());
        assert!(Opcode::try_from(6).is_err());
        assert!(Opcode::try_from(12).is_err());
    }
}
