use super::Block;

/// `Data` is a packet that contains a 2-byte block number and up to 512
/// bytes of data.
#[derive(Debug)]
pub struct Data {
    /// The block identifier for this data.
    pub block: Block,

    /// The payload.
    pub data: Vec<u8>,
}
