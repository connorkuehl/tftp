use super::mode::Mode;

mod rrq;
mod wrq;

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
