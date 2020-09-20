use std::convert::TryFrom;
use std::fmt::Debug;

mod bytes;

/// `Opcode` is an identifier for a TFTP packet. It is always the first
/// two bytes of a TFTP header.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    /// Read request
    Rrq = 1,

    /// Write request
    Wrq = 2,

    /// Data packet
    Data = 3,

    /// Acknowledgement packet
    Ack = 4,

    /// Error packet
    Error = 5,
}

/// `Mode` represents a desired transmission mode for a TFTP transfer. It
/// is used in request packets.
#[derive(Debug, PartialEq)]
pub enum Mode {
    /// Mail is obsolete and RFC 1350 states it should not be implemented
    /// or used.
    Mail,

    /// NetAscii is just 7-bit ASCII.
    NetAscii,

    /// Octet, or binary transmission.
    Octet,
}

impl From<Mode> for String {
    fn from(mode: Mode) -> String {
        match mode {
            Mode::Mail => "mail".to_string(),
            Mode::NetAscii => "netascii".to_string(),
            Mode::Octet => "octet".to_string(),
        }
    }
}

/// `ErrorCode` represents the error conditions that can be reached during
/// a regular TFTP operation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorCode {
    /// Not defined, see error message (if any).
    NotDefined = 0,

    /// File not found.
    FileNotFound = 1,

    /// Access violation.
    AccessViolation = 2,

    /// Disk full or allocation exceeded.
    DiskFull = 3,

    /// Illegal TFTP operation.
    IllegalOperation = 4,

    /// Unknown transfer ID.
    UnknownTid = 5,

    /// File already exists.
    FileAlreadyExists = 6,

    /// No such user.
    NoSuchUser = 7,
}

impl From<ErrorCode> for String {
    fn from(code: ErrorCode) -> String {
        match code {
            ErrorCode::NotDefined => "Not defined".to_string(),
            ErrorCode::FileNotFound => "File not found".to_string(),
            ErrorCode::AccessViolation => "Access violation".to_string(),
            ErrorCode::DiskFull => "Disk full or allocation exceeded".to_string(),
            ErrorCode::IllegalOperation => "Illegal TFTP operation".to_string(),
            ErrorCode::UnknownTid => "Unknown transfer ID".to_string(),
            ErrorCode::FileAlreadyExists => "File already exists".to_string(),
            ErrorCode::NoSuchUser => "No such user".to_string(),
        }
    }
}

pub type Block = u16;

/// `Rq` is a request packet (either read or write) and it identifies the
/// object/filename that will be uploaded/downloaded as well as the mode it
/// should be transferred in.
#[derive(Debug)]
pub struct Rq {
    /// The filename to operate on.
    pub filename: String,

    /// The mode for the TFTP transmission.
    pub mode: Mode,
}

/// `Data` is a packet that contains a 2-byte block number and up to 512
/// bytes of data.
#[derive(Debug)]
pub struct Data {
    /// The block identifier for this data.
    pub block: Block,

    /// The payload.
    pub data: Vec<u8>,
}

/// An `Ack` packet acknowledges successful receipt of a `Data` packet
/// and indicates that the next `Data` packet should be sent.
#[derive(Debug)]
pub struct Ack {
    /// The block identifier that is being acknowledged.
    pub block: Block,
}

/// An `Error` packet is a courtesy packet that is sent prior to terminating
/// the TFTP connection due to an unrecoverable error.
#[derive(Debug)]
pub struct Error {
    /// An integer code that describes the error.
    pub code: ErrorCode,

    /// A human readable description of the error.
    pub message: String,
}

/// A TFTP packet.
#[derive(Debug)]
pub struct Packet<T: Debug + Into<Vec<u8>> + TryFrom<Vec<u8>>> {
    /// TFTP packet identifier.
    pub header: Opcode,

    /// The contents of the packet.
    pub body: T,
}

impl Packet<Rq> {
    pub fn read(filename: String, mode: Mode) -> Packet<Rq> {
        let header = Opcode::Rrq;
        let body = Rq {
            filename,
            mode,
        };

        Packet { header, body }
    }

    pub fn write(filename: String, mode: Mode) -> Packet<Rq> {
        let header = Opcode::Wrq;
        let body = Rq {
            filename,
            mode,
        };

        Packet { header, body }
    }
}

impl Packet<Data> {
    pub fn new(block: Block, data: Vec<u8>) -> Packet<Data> {
        let header = Opcode::Data;
        let body = Data {
            block,
            data,
        };

        Packet { header, body }
    }
}

impl Packet<Ack> {
    pub fn new(block: Block) -> Packet<Ack> {
        let header = Opcode::Ack;
        let body = Ack {
            block,
        };

        Packet { header, body }
    }
}

impl Packet<Error> {
    pub fn new(code: ErrorCode) -> Packet<Error> {
        let header = Opcode::Error;
        let body = Error {
            code,
            message: code.into(),
        };

        Packet { header, body }
    }

    pub fn with_custom_message(code: ErrorCode, message: String) -> Packet<Error> {
        let header = Opcode::Error;
        let body = Error {
            code,
            message,
        };

        Packet { header, body }
    }
}

#[cfg(test)]
mod tests {
}
