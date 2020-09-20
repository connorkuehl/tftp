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

#[cfg(test)]
mod tests {
}
