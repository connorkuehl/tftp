use super::Block;

/// An `Ack` packet acknowledges successful receipt of a `Data` packet
/// and indicates that the next `Data` packet should be sent.
#[derive(Debug)]
pub struct Ack {
    /// The block identifier that is being acknowledged.
    pub block: Block,
}
