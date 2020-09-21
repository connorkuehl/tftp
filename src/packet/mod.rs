mod ack;
mod data;
mod error;
mod mode;
mod opcode;
mod rq;

mod sealed {
    use crate::bytes::{FromBytes, IntoBytes};
    use super::opcode::Opcode;

    pub trait Packet: FromBytes + IntoBytes {
        const OPCODE: Opcode;
    }
}

pub type Block = u16;
