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
