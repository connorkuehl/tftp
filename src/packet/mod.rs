mod bytes;

/// `Opcode` is an identifier for a TFTP packet. It is always the first
/// two bytes of a TFTP header.
#[derive(Debug, PartialEq)]
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

/// `ErrorCode` represents the error conditions that can be reached during
/// a regular TFTP operation.
#[derive(Debug, PartialEq)]
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

type Block = u16;

/// `Rq` is a request packet (either read or write) and it identifies the
/// object/filename that will be uploaded/downloaded as well as the mode it
/// should be transferred in.
pub struct Rq {
    /// The filename to operate on.
    pub filename: String,

    /// The mode for the TFTP transmission.
    pub mode: Mode,
}

/// `Data` is a packet that contains a 2-byte block number and up to 512
/// bytes of data.
pub struct Data {
    /// The block identifier for this data.
    pub block: Block,

    /// The payload.
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
}
