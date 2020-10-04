//! A utility trait for attempting to parse a desired Packet or producing
//! an error.

use std::io::Result;
use std::net::UdpSocket;

use super::Packet;
use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::{error, Error};

/// Implementors can attempt to produce a packet of a certain type from
/// the provided bytes.
///
/// ## Remarks
///
/// Note that implementors may introduce side effects.
pub trait ExpectPacket {
    /// Tries to produce the desired packet type.
    fn expect_packet<P: super::sealed::Packet, B: AsRef<[u8]>>(
        &self,
        bytes: B,
    ) -> Result<Packet<P>>;
}

impl ExpectPacket for UdpSocket {
    fn expect_packet<P: super::sealed::Packet, B: AsRef<[u8]>>(
        &self,
        bytes: B,
    ) -> Result<Packet<P>> {
        let bytes = bytes.as_ref();
        match Packet::<P>::from_bytes(&bytes) {
            // Yay
            Ok(packet) => Ok(packet),
            Err(_) => {
                // If we didn't get the packet we were expecting, maybe the
                // peer sent us an error packet.
                if let Ok(err_pkt) = Packet::<Error>::from_bytes(&bytes) {
                    Err(err_pkt.into())
                } else {
                    // Peer didn't send us the expected packet OR an error
                    // packet. Send them our own error packet and terminate
                    // the connection.
                    let kind = error::Code::IllegalOperation;
                    let err = Packet::error(kind, kind.as_str());
                    let bytes = err.clone().into_bytes();
                    let _ = self.send(&bytes[..]);
                    Err(err.into())
                }
            }
        }
    }
}
