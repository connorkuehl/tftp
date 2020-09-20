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

#[cfg(test)]
mod tests {
}
